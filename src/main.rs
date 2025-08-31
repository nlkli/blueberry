mod config;
mod core;
mod ctxdata;
mod error;
mod llmprovider;
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

        while let Some(i) = rx.recv().await {
            println!("got new feedback: {:?}", i);
        }
    }
}

#[tokio::main]
async fn main() {
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

            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { scli.get_review_count().await.unwrap() })
            });

            Ok(result)
        })
        .unwrap();

    table.set("data", table_data).unwrap();
    table.set("fn", lua_fn).unwrap();

    lua.globals().set("ctx", table).unwrap();

    let _v = lua.load(&code).exec().unwrap();

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

    // let new_reviews = scli
    //     .get_last_new_reviews(
    //         2,
    //         (time::UtcDateTime::now().unix_timestamp() as u64) - 1800 * 2,
    //     )
    //     .await
    //     .unwrap();

    // dbg!(new_questions);

    // dbg!(new_reviews);

    // return;

    loop {
        let (tx, mut rx) = mpsc::unbounded_channel();

        scli.spawn_new_feedback_observer(tx, Duration::from_secs(11), Duration::from_secs(7));

        while let Some(i) = rx.recv().await {
            println!("got new feedback: {:?}", i);
        }
    }

    return;

    let cli = WbSellerClient::from_env();

    let _x = serde_json::from_str::<()>("").unwrap();

    let review_count = cli.get_review_count_unanswered().await.unwrap();

    println!("{:?}", review_count);

    return;

    let new_feedbacks_questions = cli.get_new_feedbacks().await.unwrap();

    // let question_list = cli
    //     .get_question_list(&wbmodels::params::QuestionListFilter {
    //         is_answered: true,
    //         take: 1000,
    //         skip: 0,
    //         ..Default::default()
    //     })
    //     .await
    //     .unwrap();

    println!("{:#?}", new_feedbacks_questions);

    return;

    let cli = Arc::new(WbSellerClient::from_env());

    let product_ctx_data = ctxdata::ProductCtxData::from_wb_product_nmid(cli, 291118780)
        .await
        .unwrap();

    // println!("{:#?}", product_ctx_data);

    let mut ctx = Context::new();

    product_ctx_data.insert_to_ctx(&mut ctx);
    ctx.insert("message", "Сколько стоит?");

    let template = std::fs::read_to_string("templates/base.j2").unwrap();

    let text = Tera::one_off(&template, &ctx, false).unwrap();

    println!("{}", text);

    // let filter = wbmodels::params::Filter {
    //     with_photo: Some(-1),
    //     // imt_id: Some(291118780),
    //     text_search: Some("291118780"),
    //     // object_ids: Some(&[291118780]),
    //     ..Default::default()
    // };

    // let res = cli.get_cards_list(Some(&filter), 100).await.unwrap();

    // let photo_url = res.cards[0].photos[0].big.clone();

    // let basket_urlpath = photo_url.split_once("/images").unwrap().0;

    // let rich_content = cli
    //     .get_product_rich_content(basket_urlpath, 1)
    //     .await
    //     .unwrap();

    // let products_price = cli
    //     .get_products_price(10, None, Some(291118780))
    //     .await
    //     .unwrap();

    // println!("{:#?}", res);

    // println!("\n\n{}", rich_content);

    // println!("\n\n{:#?}", products_price);

    // webapp::run("0.0.0.0:3344").await.unwrap();
    // let cli = Arc::new(OzonSellerClient::new(
    //     "826115".into(),
    //     "62cf0574-910b-46d4-af17-da2bea0c541e".into(),
    // ));

    // let res = cli.get_review_list(33, None, None).await.unwrap();

    // println!("{:#?}", res);

    // let product_ctx_data = ctxdata::ProductCtxData::from_ozon_product_sku(cli, 1766876847)
    //     .await
    //     .unwrap();

    // let mut ctx = Context::new();

    // product_ctx_data.insert_to_ctx(&mut ctx);
    // ctx.insert("message", "Сколько стоит?");

    // let template = std::fs::read_to_string("templates/base.j2").unwrap();

    // let text = Tera::one_off(&template, &ctx, false).unwrap();

    // println!("{}", text);

    //------------------------------------

    // let filter = models::params::Filter {
    //     sku: Some(&[2470406780]),
    //     ..Default::default()
    // };

    // let product_info_list = cli.get_product_info_list(&filter).await.unwrap();

    // println!("{:#?}", product_info_list.items);

    // let sku = product_info_list.items[0].sku;
    // let product_id = product_info_list.items[0].id;

    // let description = cli.get_product_info_description(product_id).await.unwrap();

    // println!("description: {}", description.result.description);

    // let product_attributes = cli
    //     .get_product_attributes_v4(Some(&filter), 1, None)
    //     .await
    //     .unwrap();

    // println!("product_attributes: {:#?}", product_attributes);

    // let description_category_id = product_attributes.result[0].description_category_id;
    // let type_id = product_attributes.result[0].type_id;

    // let category_attributes = cli
    //     .get_attributes(description_category_id, None, type_id)
    //     .await
    //     .unwrap()
    //     .result;

    // let mut attributes = HashMap::new();

    // for attr in &product_attributes.result[0].attributes {
    //     if let Some(find) = category_attributes.iter().find(|v| v.id == attr.id) {
    //         attributes.insert(
    //             find.name.clone(),
    //             attr.values
    //                 .iter()
    //                 .map(|v| v.value.as_str())
    //                 .collect::<Vec<_>>()
    //                 .join(", "),
    //         );
    //     }
    // }

    // println!("attributes: {:#?}", attributes);

    // let product = Product {
    //     id: sku.to_string(),
    //     name: product_info_list.items[0].name.clone(),
    //     price: product_info_list.items[0].marketing_price.clone(),
    // };

    // let template = std::fs::read_to_string("prompts/base.j2").unwrap();

    // ctx.insert("product", &product);

    // let text = Tera::one_off(&template, &ctx, true).unwrap();

    // println!("{}", text);
}
