use dynomite::{
    attr_map,
    dynamodb::{
        AttributeDefinition, CreateTableInput, DynamoDb, DynamoDbClient, GetItemInput,
        KeySchemaElement, ProvisionedThroughput, PutItemInput, ScanInput,
    },
    retry::Policy,
    Attributes, DynamoDbExt, Item, Retries,
};
use futures::{future, TryStreamExt};
#[cfg(feature = "default")]
use rusoto_core_default::Region;
#[cfg(feature = "rustls")]
use rusoto_core_rustls::Region;
use std::{convert::TryFrom, error::Error};
use uuid::Uuid;

#[derive(Attributes, Debug, Clone)]
pub struct Author {
    id: Uuid,
    #[dynomite(default)]
    name: String,
}

#[derive(Item, Debug, Clone)]
pub struct Book {
    #[dynomite(partition_key)]
    id: Uuid,
    #[dynomite(rename = "bookTitle")]
    title: String,
    authors: Option<Vec<Author>>,
}

/// create a book table with a single string (S) primary key.
/// if this table does not already exists
/// this may take a second or two to provision.
/// it will fail if this table already exists but that's okay,
/// this is just an example :)
async fn bootstrap<D>(
    client: &D,
    table_name: String,
) where
    D: DynamoDb,
{
    let _ = client
        .create_table(CreateTableInput {
            table_name,
            key_schema: vec![KeySchemaElement {
                attribute_name: "id".into(),
                key_type: "HASH".into(),
            }],
            attribute_definitions: vec![AttributeDefinition {
                attribute_name: "id".into(),
                attribute_type: "S".into(),
            }],
            provisioned_throughput: Some(ProvisionedThroughput {
                read_capacity_units: 1,
                write_capacity_units: 1,
            }),
            ..CreateTableInput::default()
        })
        .await;
}

// this will create a rust book shelf in your aws account!
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    // create rusoto client
    let client = DynamoDbClient::new(Region::default()).with_retries(Policy::default());

    let table_name = "books".to_string();

    bootstrap(&client, table_name.clone()).await;

    let authors = Some(vec![Author {
        id: Uuid::new_v4(),
        name: "Jo Bloggs".into(),
    }]);

    let book = Book {
        id: Uuid::new_v4(),
        title: "rust".into(),
        authors,
    };

    // print the key for this book
    // requires bringing `dynomite::Item` into scope
    println!("book.key() {:#?}", book.key());

    // add a book to the shelf
    println!(
        "put_item() result {:#?}",
        client
            .put_item(PutItemInput {
                table_name: table_name.clone(),
                item: book.clone().into(), // <= convert book into it's attribute map representation
                ..PutItemInput::default()
            })
            .await?
    );

    println!(
        "put_item() result {:#?}",
        client
            .put_item(PutItemInput {
                table_name: table_name.clone(),
                // convert book into it's attribute map representation
                item: Book {
                    id: Uuid::new_v4(),
                    title: "rust and beyond".into(),
                    authors: Some(vec![Author {
                        id: Uuid::new_v4(),
                        name: "Jim Ferris".into(),
                    }]),
                }
                .into(),
                ..PutItemInput::default()
            })
            .await?
    );

    // scan through all pages of results in the books table for books who's title is "rust"
    println!(
        "scan result {:#?}",
        client
            .clone()
            .scan_pages(ScanInput {
                limit: Some(1), // to demonstrate we're getting through more than one page
                table_name: table_name.clone(),
                filter_expression: Some("bookTitle = :title".into()),
                expression_attribute_values: Some(attr_map!(
                    ":title" => "rust".to_string()
                )),
                ..ScanInput::default()
            })
            .try_for_each(|item| {
                println!("stream_scan() item {:#?}", Book::try_from(item));
                future::ready(Ok(()))
            })
            .await? // attempt to convert a attribute map to a book type
    );

    // get the "rust' book by the Book type's generated key
    println!(
        "get_item() result {:#?}",
        client
            .get_item(GetItemInput {
                table_name,
                key: book.key(), // get a book by key
                ..GetItemInput::default()
            })
            .await?
            .item
            .map(Book::try_from) // attempt to convert a attribute map to a book type
    );
    Ok(())
}
