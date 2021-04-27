use scrapman::{
    stage::ScrapeStage, ClickElement, FillElement, JsonValue, OpenUrl, QueryElement, ScrapePipeline, Scrapman,
    Selector, SetModelAttribute, StoreModel, Value,
};
use std::{error::Error, fs::read_to_string};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let values = serde_yaml::from_str::<JsonValue>(&read_to_string("data/cian.yaml")?)?;

    let pipeline = ScrapePipeline::default()
        .push(OpenUrl::new(Value::Context("start_url".into())))
        .push(QueryElement::global(Selector::Id, Value::Context("search.input_id".into())))
        .push(FillElement::new(Value::Context("search.query".into())))
        .push(QueryElement::global(
            Selector::LinkText,
            Value::Context("search.button_text".into()),
        ))
        .push(ClickElement)
        .push(
            ScrapeStage::from(
                QueryElement::global(Selector::Css, Value::Context("selectors.card".into())).for_each(
                    ScrapePipeline::default()
                        .push(QueryElement::scoped(Selector::Css, Value::Context("selectors.title".into())))
                        .push(SetModelAttribute::new("title", Value::ElementText))
                        .push(QueryElement::scoped(Selector::Css, Value::Context("selectors.price".into())))
                        .push(SetModelAttribute::new("price", Value::ElementText))
                        .push(SetModelAttribute::new("class", Value::ElementAttribute("class".into())))
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
