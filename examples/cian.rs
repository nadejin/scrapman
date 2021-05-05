use log::error;
use scrapman::{
    ClickElement, FillElement, FlowControl, JsonValue, OpenUrl, QueryElement, ScrapePipeline, ScrapeStage, Scrapman,
    Selector, SetModelAttribute, StoreModel, Value,
};
use std::{error::Error, fs::read_to_string};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init_timed();

    let values = serde_yaml::from_str::<JsonValue>(&read_to_string("data/cian.yaml")?)?;

    let pipeline = ScrapePipeline::default()
        // Entry point
        .push(OpenUrl::new(Value::context("start_url")))
        // Main search input element is queried
        .push(QueryElement::global(Selector::Id, Value::context("search.input_id")))
        // The search input is filled with the value from configuration file
        .push(FillElement::new(Value::context("search.query")))
        // The submit button is queried
        .push(QueryElement::global(
            Selector::LinkText,
            Value::context("search.button_text"),
        ))
        // The submit button is clicked
        .push(ClickElement)
        // Search results card elements are queried, and a nested pipeline is launched for every found card
        .push(
            ScrapeStage::from(
                QueryElement::global(Selector::Css, Value::context("selectors.card")).for_each(
                    ScrapePipeline::default()
                        // Title containing element is queried
                        .push(QueryElement::scoped(Selector::Css, Value::context("selectors.title")))
                        // Title element text value is stored in the context with "title" key
                        .push(SetModelAttribute::new("title", Value::ElementText))
                        // Price containing element is queried
                        .push(QueryElement::scoped(Selector::Css, Value::context("selectors.price")))
                        // Price element text value is stored in the context with "price" key
                        .push(SetModelAttribute::new("price", Value::ElementText))
                        // All populated attributes are stored as a model and removed from the context
                        .push(StoreModel),
                ),
            )
            .with_name("QueryCards"),
        )
        // Next search page element is queried, if not found - scrape pipeline quits
        .push(
            ScrapeStage::from(QueryElement::global(
                Selector::Css,
                Value::context("selectors.next_page"),
            ))
            .on_any_error(FlowControl::Quit),
        )
        // Next search page element is clicked
        .push(ScrapeStage::from(ClickElement))
        // Preload overlay element is queried with some interval, while it exists - the pipeline is not moving further
        // Once the loading is complete, and the overlay is removed from the page, the pipeline returns to the query cards stage
        .push(
            ScrapeStage::from(QueryElement::global(Selector::Css, Value::context("selectors.preload")))
                .on_complete(FlowControl::repeat_with_delay(0.5))
                // TODO: error specific handlers
                .on_any_error(FlowControl::goto("QueryCards")),
        );

    let scrapman = Scrapman::new("http://localhost:4444");

    match scrapman.launch(pipeline, values).await {
        Ok(ctx) => (),
        Err(error) => error!("Error: {}", error),
    };

    Ok(())
}
