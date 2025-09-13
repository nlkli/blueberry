mod config;
mod controller;
mod db;
mod error;
mod genai;
mod sellerapi;
mod webapp;
use error::Result;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
    time::Duration,
};

use serde::Serialize;
use tera::{Context, Tera};
use tokio::sync::mpsc;

use crate::sellerapi::{
    OzonSellerClient, SellerClient, WbSellerClient, abcmodels, ozmodels, wbmodels,
};

async fn product_ai_summary(scli: SellerClient, template_path: &str, ai_model: &str) {
    let mut rx = scli.all_products_stream();

    let template = std::fs::read_to_string(template_path).unwrap();

    let provider = genai::AiProvider::from_env();

    let mut n = 1;
    while let Some(res) = rx.recv().await {
        if let Err(e) = res {
            eprintln!("Ошибка запроса всех товаров: {e}");
            return;
        }

        let product = unsafe { res.unwrap_unchecked() };

        scli.get_product_format_info(&product.id).await;

        let res = scli.get_product_format_info(&product.id).await;

        if let Err(e) = res {
            eprintln!("Ошибка формирования данных контекста товара для шаблона: {e}");
            return;
        }

        let product_ctx_data = unsafe { res.unwrap_unchecked() };

        let mut ctx = Context::new();
        ctx.insert("product", &product_ctx_data);
        let res = Tera::one_off(&template, &ctx, false);

        if let Err(e) = res {
            eprintln!("Ошибка применения текстового шаблона: {e}");
            return;
        }

        let prompt = unsafe { res.unwrap_unchecked() };

        let product_ai_summary_id = format!("{}/{}", scli.str_symbol(), product.id);

        println!("{n}. {product_ai_summary_id} ai_summary:");

        let res = provider.send_prompt(&prompt, ai_model).await;

        if let Err(e) = res {
            eprintln!("Ошибка обращения к AI провайдеру: {e}");
            return;
        }

        let mut chat_response = unsafe { res.unwrap_unchecked() };

        let summary = chat_response
            .take_message(0)
            .map(|v| v.content)
            .unwrap_or_default();

        if let Err(e) = db::insert_or_replace_product_ai_summary(&product_ai_summary_id, &summary) {
            eprintln!("Ошибка записи в базу данных: {e}");
            return;
        }

        println!("{summary}");
        println!("------------------------------------------------");

        n += 1;
    }
}

async fn run_feedback_observer(scli: SellerClient) {
    if let SellerClient::Ozon(ref cli) = scli {
        if !cli
            .seller_rating_summary()
            .await
            .map(|r| r.premium_plus)
            .inspect_err(|e| eprintln!("Не удалось получить информацию о клиенте Ozon Seller: {e}"))
            .unwrap_or_default()
        {
            println!(
                "Методы работы с вопросами и отзывами доступны только для продавцов с подпиской Premium Plus."
            );
            return;
        }
    }

    let provider = genai::AiProvider::from_env();

    let q_template = std::fs::read_to_string("templates/question.j2").unwrap();

    loop {
        println!("Запуск обработчика обратной связи...");

        let mut rx =
            scli.spawn_new_feedback_observer(Duration::from_secs(11), Duration::from_secs(7));

        while let Some(res) = rx.recv().await {
            if let Err(e) = res {
                eprintln!("Ошибка получения новой обратной связи: {e}");
                break;
            }

            let feedback = unsafe { res.unwrap_unchecked() };

            match feedback {
                abcmodels::NewFeedback::Question(q) => {
                    println!("{:#?}", q);

                    let s = db::select_product_ai_summary(&format!("{}/{}", "wb", q.product_id))
                        .unwrap()
                        .unwrap();

                    let mut ctx = Context::new();

                    ctx.insert("question", &q.text);
                    ctx.insert("ai_summary", &s.ai_summary);

                    let prompt = Tera::one_off(&q_template, &ctx, false).unwrap();

                    let mut chat_response = provider
                        .send_prompt(&prompt, "deepseek/deepseek-r1-0528:free")
                        .await
                        .unwrap();

                    let answer = chat_response.take_message(0).unwrap().content;

                    println!("Ответ на вопрос: {}", answer);
                }
                abcmodels::NewFeedback::Review(r) => {
                    println!("{:#?}", r);
                }
            }
        }
    }
}

async fn test_question(scli: SellerClient) {
    if let SellerClient::Ozon(ref cli) = scli {
        if !cli
            .seller_rating_summary()
            .await
            .map(|r| r.premium_plus)
            .inspect_err(|e| eprintln!("Не удалось получить информацию о клиенте Ozon Seller: {e}"))
            .unwrap_or_default()
        {
            println!(
                "Методы работы с вопросами и отзывами доступны только для продавцов с подпиской Premium Plus."
            );
            return;
        }
    }

    let template = std::fs::read_to_string("templates/question.j2").unwrap();

    loop {
        let mut product_id = String::new();
        println!("Enter productID: ");
        std::io::stdin().read_line(&mut product_id).unwrap();

        let ai_summary =
            db::select_product_ai_summary(&format!("{}/{}", scli.str_symbol(), product_id.trim()))
                .unwrap()
                .unwrap()
                .ai_summary;

        let mut question = String::new();
        println!("Enter your question: ");
        std::io::stdin().read_line(&mut question).unwrap();

        let mut ctx = Context::new();

        ctx.insert("question", question.trim());
        ctx.insert("ai_summary", &ai_summary);

        let prompt = Tera::one_off(&template, &ctx, false).unwrap();

        println!("Ожидание ответа AI провайдера...");
        let mut chat_response = genai::AiProvider::from_env()
            .send_prompt(&prompt, "deepseek/deepseek-r1-0528:free")
            .await
            .unwrap();

        let answer = chat_response.take_message(0).unwrap().content;

        println!("\nAnswer to question:\n{}", answer);
    }
}

#[tokio::main]
async fn main() {
    let cli = Arc::new(WbSellerClient::from_env());

    let scli = SellerClient::Wb(cli.clone());

    test_question(scli).await;

    return;

    // deepseek/deepseek-r1-0528:free
    // deepseek/deepseek-chat-v3-0324:free
    // deepseek/deepseek-chat-v3.1:free
    // deepseek/deepseek-r1:free

    run_feedback_observer(scli.clone()).await;

    // product_ai_summary(
    //     scli,
    //     "templates/product_summary.j2",
    //     "deepseek/deepseek-r1-0528:free",
    // )
    // .await;

    return;
}
