use std::collections::HashSet;
use yew::{Callback, classes, function_component, Html, html, TargetCast, use_reducer, use_state};
use serde::Serialize;
use web_sys::{HtmlInputElement, InputEvent};
use yew_hooks::use_set;
use yew_custom_components::pagination::Pagination;
use yew_custom_components::table::{Options, Table};
use yew_custom_components::table::types::{ColumnBuilder, TableData};



#[function_component(Home)]
pub fn home() -> Html {
    // Mock data holder
    let data = use_reducer(crate::types::mock_data::Data::default);
    let mock_data = (*data).clone();

    // Search term
    let search_term = use_state(|| None::<String>);
    let search = (*search_term).as_ref().cloned();

    let page=use_state(||0usize);
    let current_page=(*page).clone();

    // Sum data
    let selected_indexes = use_set(HashSet::<usize>::new());
    let selected = selected_indexes.current().clone();

    let sum = mock_data.data.iter().enumerate().fold(0, |acc, (index, (_, _, value))| {
        if selected.contains(&index) {
            acc + value
        } else {
            acc
        }
    });

    // Column definition
    let columns = vec![
        ColumnBuilder::new("select").orderable(true).short_name("Sum").data_property("select").header_class("user-select-none").build(),
        ColumnBuilder::new("id").orderable(true).short_name("ID").data_property("id").header_class("user-select-none").build(),
        ColumnBuilder::new("name").orderable(true).short_name("Name").data_property("name").header_class("user-select-none").build(),
        ColumnBuilder::new("value").orderable(true).short_name("Value").data_property("value").header_class("user-select-none").build(),
    ];


    // Table options
    let options = Options {
        unordered_class: Some("fa-sort".to_string()),
        ascending_class: Some("fa-sort-up".to_string()),
        descending_class: Some("fa-sort-down".to_string()),
        orderable_classes: vec!["mx-1".to_string(), "fa-solid".to_string()],
    };

    // Handle sum
    let callback_sum = {
        let selected_indexes = selected_indexes.clone();
        Callback::from(move |index: usize| {
            if !selected_indexes.insert(index) {
                selected_indexes.remove(&index);
            }
        })
    };

    // Fill the table data structure with actual data
    let mut table_data = Vec::new();
    for (index, (id, name, value)) in mock_data.data.iter().enumerate() {
        table_data.push(TableLine {
            original_index: index,
            id: *id,
            name: name.clone(),
            value: *value,
            checked: selected.contains(&index),
            sum_callback: callback_sum.clone(),
        })
    }

    // Handle search input
    let oninput_search = {
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if input.value().is_empty() {
                search_term.set(None);
            } else {
                search_term.set(Some(input.value()));
            }
        })
    };

    let pagination_options = yew_custom_components::pagination::Options::default()
        .show_prev_next(true)
        .list_classes(vec!(String::from("pagination")))
        .item_classes(vec!(String::from("page-item")))
        .link_classes(vec!(String::from("page-link")))
        .active_item_classes(vec!(String::from("active")))
        .disabled_item_classes(vec!(String::from("disabled")));

    let handle_page = {
        let page = page.clone();
        Callback::from(move |id: usize| {
            page.set(id);
        })
    };

    html!(
        <>
            <h1>{"Minimal table Example"}</h1>
            <div class="flex-grow-1 p-2 input-group mb-2">
                <span class="input-group-text">
                    <i class="fas fa-search"></i>
                </span>
                <input class="form-control" type="text" id="search" placeholder="Search" oninput={oninput_search} />
            </div>
            <Table<TableLine> options={options.clone()} limit={Some(10)} page={current_page} search={search.clone()} classes={classes!("table", "table-hover")} columns={columns.clone()} data={table_data.clone()} orderable={true}/>
            <Pagination total={table_data.len()} limit={10} options={pagination_options} on_page={Some(handle_page)}/>
            <h5>{"Sum of selected"} <span class="badge text-bg-secondary">{sum}</span></h5>
        </>
    )
}

#[derive(Clone, Serialize, Debug, Default)]
struct TableLine {
    pub original_index: usize,
    pub id: i32,
    pub name: String,
    pub value: i64,
    pub checked: bool,
    #[serde(skip_serializing)]
    pub sum_callback: Callback<usize>,
}

impl PartialEq<Self> for TableLine {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value == other.value && self.checked == other.checked
    }
}

impl PartialOrd for TableLine {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl TableData for TableLine {
    fn get_field_as_html(&self, field_name: &str) -> yew_custom_components::table::error::Result<Html> {
        match field_name {
            "select" => Ok(html!( <input type="checkbox" checked={self.checked}
                onclick={
                let value = self.original_index;
                let handle_sum = self.sum_callback.clone();
                move |_| { handle_sum.emit(value); }
                } /> )
            ),
            "id" => Ok(html! { self.id }),
            "name" => Ok(html! { self.name.clone() }),
            "value" => Ok(html! { self.value }),
            _ => Ok(html! {}),
        }
    }

    fn get_field_as_value(&self, field_name: &str) -> yew_custom_components::table::error::Result<serde_value::Value> {
        match field_name {
            "id" => Ok(serde_value::Value::I32(self.id)),
            "name" => Ok(serde_value::Value::String(self.name.clone())),
            "value" => Ok(serde_value::Value::I64(self.value)),
            "select" => Ok(serde_value::Value::Bool(self.checked)),
            _ => Ok(serde_value::to_value(()).unwrap()),
        }
    }

    fn matches_search(&self, needle: Option<String>) -> bool {
        match needle {
            Some(needle) => self.name.to_lowercase().contains(&needle.to_lowercase()),
            None => true,
        }
    }
}
