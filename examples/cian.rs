use scrapman::{
    stage::ScrapeStage, ClickElement, FillElement, JsonValue, OpenUrl, QueryElement, ScrapePipeline, Scrapman,
    Selector, SetModelAttribute, StoreModel, Value,
};
use std::{error::Error, fs::read_to_string};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let values = serde_yaml::from_str::<JsonValue>(&read_to_string("data/cian.yaml")?)?;

    let pipeline = ScrapePipeline::default()
        .push(OpenUrl::new(Value::context("start_url")))
        .push(QueryElement::global(Selector::Id, Value::context("search.input_id")))
        .push(FillElement::new(Value::context("search.query")))
        .push(QueryElement::global(
            Selector::LinkText,
            Value::context("search.button_text"),
        ))
        .push(ClickElement)
        .push(
            ScrapeStage::from(
                QueryElement::global(Selector::Css, Value::context("selectors.card")).for_each(
                    ScrapePipeline::default()
                        .push(QueryElement::scoped(Selector::Css, Value::context("selectors.title")))
                        .push(SetModelAttribute::new("title", Value::ElementText))
                        .push(QueryElement::scoped(Selector::Css, Value::context("selectors.price")))
                        .push(SetModelAttribute::new("price", Value::ElementText))
                        .push(StoreModel),
                ),
            )
            .with_name("QueryCards"),
        );

    println!("{}", serde_yaml::to_string(&pipeline)?);

    let scrapman = Scrapman::new("http://localhost:4444");

    match scrapman.launch(pipeline, values).await {
        Ok(ctx) => println!("Scraped data:\n{}", serde_yaml::to_string(&ctx.models)?),
        Err(error) => println!("Error: {}", error),
    };

    Ok(())
}
