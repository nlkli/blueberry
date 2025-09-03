mod config;
mod core;
mod ctxdata;
mod db;
mod error;
mod genai;
mod observer;
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

use crate::sellerapi::{OzonSellerClient, SellerClient, WbSellerClient, ozmodels, wbmodels};

static AsyncRuntime: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().unwrap());

async fn run_feedback_observer() {
    let cli = Arc::new(WbSellerClient::from_env());

    let scli = SellerClient::Wb(cli);

    loop {
        let (tx, mut rx) = mpsc::unbounded_channel();

        scli.spawn_new_feedback_observer(tx, Duration::from_secs(11), Duration::from_secs(7));

        let mut n = 0;
        while let Some(i) = rx.recv().await {
            println!("got new feedback: {:?}", i);
            n += 1;
        }
        println!("{}", n);
    }
}

#[tokio::main]
async fn main() {
    let cli = Arc::new(WbSellerClient::from_env());

    let scli = SellerClient::Wb(cli);

    let (tx, mut rx) = mpsc::unbounded_channel();

    scli.all_products_stream(tx);

    let mut n = 0;
    while let Some(i) = rx.recv().await {
        let pd = i.unwrap();

        let product_ctx_data = ctxdata::ProductCtxData::new(&scli, &pd.id).await.unwrap();

        // println!("{:#?}", product_ctx_data);

        let mut ctx = Context::new();

        product_ctx_data.insert_to_ctx(&mut ctx);

        let template = std::fs::read_to_string("templates/product_summary.j2").unwrap();

        let text = Tera::one_off(&template, &ctx, false).unwrap();

        let provider = genai::AiProvider::new(
            "https://openrouter.ai/api/v1",
            "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",
        );

        let mut res = provider
            .chat(&genai::ChatRequest {
                model: "deepseek/deepseek-r1-0528:free".into(),
                messages: vec![genai::Message {
                    role: "user".into(),
                    content: text,
                }],
                ..Default::default()
            })
            .await
            .unwrap();

        let text = res.take_message(0).map(|v| v.content).unwrap_or_default();

        db::insert_or_replace_product_ai_summary(&format!("{}/{}", pd.place_symbol, pd.id), &text);
    }
    println!("{}", n);

    return;
    let cli = Arc::new(WbSellerClient::from_env());

    let scli = SellerClient::Wb(cli);

    let product_ctx_data = ctxdata::ProductCtxData::new(&scli, "238348161")
        .await
        .unwrap();

    // println!("{:#?}", product_ctx_data);

    let mut ctx = Context::new();

    product_ctx_data.insert_to_ctx(&mut ctx);
    ctx.insert("message", "Сколько стоит?");

    let template = std::fs::read_to_string("templates/product_summary.j2").unwrap();

    let text = Tera::one_off(&template, &ctx, false).unwrap();

    let provider = genai::AiProvider::new(
        "https://openrouter.ai/api/v1",
        "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",
    );

    let mut res = provider
        .chat(&genai::ChatRequest {
            model: "deepseek/deepseek-r1-0528:free".into(),
            messages: vec![genai::Message {
                role: "user".into(),
                content: text,
            }],
            ..Default::default()
        })
        .await
        .unwrap();

    println!(
        "{}",
        res.take_message(0).map(|v| v.content).unwrap_or_default()
    );

    return;

    let mut v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];

    v.sort_by(|a, b| a.cmp(a));

    println!("{:?}", v);

    return;
    run_feedback_observer().await;

    return;

    let cli = Arc::new(WbSellerClient::from_env());

    let scli = SellerClient::Wb(cli);

    // let new_questions = scli
    //     .get_last_new_questions(
    //         2,
    //         (time::UtcDateTime::now().unix_timestamp() as u64) - 1800 * 2,
    //     )
    //     .await
    //     .unwrap();

    let new_reviews = scli.get_last_new_reviews(2, 1756744679).await.unwrap();

    println!("{:?}", new_reviews);

    return;

    let cli = genai::AiProvider::new(
        "https://openrouter.ai/api/v1",
        "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",
    );

    let res = cli
        .chat(&genai::ChatRequest {
            model: "deepseek/deepseek-r1-0528:free".into(),
            messages: vec![genai::Message {
                role: "user".into(),
                content: "Hello, my name is Nikita!!".into(),
            }],
            ..Default::default()
        })
        .await
        .unwrap();

    println!("{:#?}", res);

    return;

    for i in 0..12 {
        let id = format!("{i}");
        let text = format!("text ...{i}");
        db::insert_or_replace_product_ai_summary(&id, &text).unwrap();
    }

    let row = db::select_product_ai_summary("3").unwrap();

    println!("{row:#?}");

    return;

    run_feedback_observer().await;

    return;

    let cli = Arc::new(WbSellerClient::from_env());

    let scli = SellerClient::Wb(cli);

    // let new_questions = scli
    //     .get_last_new_questions(
    //         2,
    //         (time::UtcDateTime::now().unix_timestamp() as u64) - 1800 * 2,
    //     )
    //     .await
    //     .unwrap();

    let new_reviews = scli.get_last_new_reviews(2, 1756744679).await.unwrap();

    println!("{:?}", new_reviews);

    return;

    run_feedback_observer().await;

    return;
    let cli = Arc::new(WbSellerClient::from_env());

    let scli = SellerClient::Wb(cli);

    let lua = mlua::Lua::new();

    let code = std::fs::read_to_string("scripts/main.lua").unwrap();

    let table = lua.create_table().unwrap();

    let table_data = lua.create_table().unwrap();

    for i in 0..8 {
        table_data.set(format!("key{i}"), i).unwrap();
    }

    let lua_fn = lua
        .create_function(move |_: &mlua::Lua, message: String| {
            println!("{message}");

            // let result = tokio::task::block_in_place(|| {
            //     tokio::runtime::Handle::current()
            //         .block_on(async { scli.get_review_count().await.unwrap() })
            // });

            Ok(0)
        })
        .unwrap();

    table.set("data", table_data).unwrap();
    table.set("fn", lua_fn).unwrap();

    lua.globals().set("ctx", table).unwrap();

    let _v = lua.load(&code).exec().unwrap();

    return;
}
