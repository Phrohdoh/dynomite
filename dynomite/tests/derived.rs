use dynomite_derive::{Attribute, Item};

#[derive(Item, Default, PartialEq, Debug, Clone)]
pub struct Author {
    #[partition_key]
    name: String,
}

#[derive(Attribute, PartialEq, Debug, Clone)]
pub enum Category {
    Foo,
}

impl Default for Category {
    fn default() -> Self {
        Category::Foo
    }
}

#[derive(Item, Default, PartialEq, Debug, Clone)]
pub struct Book {
    #[partition_key]
    title: String,
    category: Category,
    authors: Option<Vec<Author>>,
}

#[derive(Item, PartialEq, Debug, Clone)]
#[dynomite(rename_all = "SCREAMING_SNAKE_CASE")]
struct Recipe {
    #[partition_key]
    #[dynomite(rename = "recipe_id")]
    id: String,
    num_servings: u64,
}

#[cfg(test)]
mod tests {

    use super::{Book, Recipe};
    use dynomite::{Attribute, Attributes, FromAttributes};

    #[test]
    fn to_and_from_book() {
        let value = Book {
            title: "rust".into(),
            ..Default::default()
        };
        let attrs: Attributes = value.clone().into();
        assert_eq!(value, Book::from_attrs(attrs).unwrap())
    }

    #[test]
    fn derive_attr() {
        #[derive(Attribute, Debug, PartialEq)]
        enum Foo {
            Bar,
        };
        assert_eq!(Foo::Bar, Foo::from_attr(Foo::Bar.into_attr()).unwrap());
    }

    #[test]
    fn rename_attributes() {
        let value = Recipe {
            id: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee".into(),
            num_servings: 2,
        };

        let attrs: Attributes = value.clone().into();

        // `id` is renamed to `recipe_id`
        assert!(attrs.contains_key("recipe_id"));
        assert!(!attrs.contains_key("id"));

        // `num_servings` is renamed to `NUM_SERVINGS`
        assert!(attrs.contains_key("NUM_SERVINGS"));
        assert!(!attrs.contains_key("num_servings"));

        assert_eq!(value, Recipe::from_attrs(attrs).unwrap());
    }
}
