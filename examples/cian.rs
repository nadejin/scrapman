use scrapman::{JsonValue, PipelineBuilder, Scrapman, Selector, Value};
use std::{error::Error, fs::read_to_string};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let values = serde_yaml::from_str::<JsonValue>(&read_to_string("data/cian.yaml")?)?;

    let pipeline = PipelineBuilder::new()
        .open_url(Value::context("start_url"))
        .find_element(Selector::Id, Value::context("search.input_id"))
        .fill_element(Value::context("search.query"))
        .find_element(Selector::LinkText, Value::context("search.button_text"))
        .click_element()
        .find_elements(
            Selector::Css,
            Value::context("selectors.card"),
            PipelineBuilder::new()
                .find_element(Selector::Css, Value::context("selectors.title"))
                .set_model_attribute("title", Value::CurrentElementText)
                .find_element(Selector::Css, Value::constant("selectors.price"))
                .set_model_attribute("price", Value::CurrentElementText)
                .store_model()
                .build(),
        )
        .build();

    println!("{}\n\n", serde_yaml::to_string(&pipeline)?);

    let scrapman = Scrapman::new("http://localhost:4444");
    if let Err(error) = scrapman.launch(pipeline, values).await {
        println!("Error: {}", error);
    }

    Ok(())
}
