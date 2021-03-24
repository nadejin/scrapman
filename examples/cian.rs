use scrapman::{PipelineBuilder, Scrapman, Selector, Value};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pipeline = PipelineBuilder::new()
        .open_url(Value::constant("https://cian.ru"))
        .find_element(Selector::Id(Value::constant("geo-suggest-input")))
        .fill_element(Value::constant("search term"))
        .find_element(Selector::LinkText(Value::constant("Найти")))
        .click_element()
        .find_elements(
            Selector::Css(Value::constant("article[data-name=\"CardComponent\"]")),
            PipelineBuilder::new()
                .find_element(Selector::Css(Value::constant("div[data-name=\"TitleComponent\"]")))
                .set_model_attribute("title", Value::ElementText)
                .find_element(Selector::Css(Value::constant("span[data-mark=\"MainPrice\"]")))
                .set_model_attribute("price", Value::ElementText)
                .store_model()
                .build(),
        )
        .build();

    println!("{}\n\n", serde_yaml::to_string(&pipeline)?);

    let scrapman = Scrapman::new("http://localhost:4444");
    if let Err(error) = scrapman.execute(pipeline).await {
        println!("Error: {}", error);
    }

    Ok(())
}
