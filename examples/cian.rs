use scrapman::{ElementSearchScope, JsonValue, PipelineBuilder, Scrapman, Selector, Value};
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
                .find_element_in(
                    Selector::Css,
                    Value::context("selectors.title"),
                    ElementSearchScope::Scoped,
                )
                .set_model_attribute("title", Value::CurrentElementText)
                .find_element_in(
                    Selector::Css,
                    Value::context("selectors.price"),
                    ElementSearchScope::Scoped,
                )
                .set_model_attribute("price", Value::CurrentElementText)
                .store_model()
                .build(),
        )
        .build();

    println!("{}\n\n", serde_yaml::to_string(&pipeline)?);

    let scrapman = Scrapman::new("http://localhost:4444");

    match scrapman.launch(pipeline, values).await {
        Ok(scraped) => println!("Scraped data:\n{}", serde_yaml::to_string(&scraped)?),
        Err(error) => println!("Error: {}", error),
    };

    Ok(())
}
