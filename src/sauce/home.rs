use std::collections::HashSet;
use yew::{Callback, classes, function_component, Html, html, TargetCast, use_reducer, use_state};
use serde::Serialize;
use web_sys::{console, HtmlInputElement, InputEvent};
use yew_hooks::use_set;
use yew_hooks::use_async;
use yew_custom_components::pagination::Pagination;
use yew_custom_components::table::{Options, Table};
use yew_custom_components::table::types::{ColumnBuilder, TableData};
use plotly::{Plot, Scatter};
use yew::prelude::*;
use serde::Deserialize;
// use cached::proc_macro::cached;


#[derive(Debug, Serialize, Deserialize)]
struct ReactionData {
    #[serde(rename = "energy")]
    energy_values: Vec<f64>,
    #[serde(rename = "cross_section")]
    cross_section_values: Vec<f64>,
}

#[derive(PartialEq, Clone, Serialize)]
pub struct XsCache {
    pub energy_values: Vec<Vec<f64>>,
    pub cross_section_values: Vec<Vec<f64>>,
    pub checkbox_selected: Vec<bool>,
}

#[derive(Properties, PartialEq)]
pub struct PlotProps {
    pub selected_indexes: HashSet<usize>,
}

#[function_component(App)]
pub fn plot_component(props: &PlotProps) -> Html {
    let selected_indexes = &props.selected_indexes;

    let p = use_async::<_, _, ()>({
        let selected_indexes = selected_indexes.clone();

        // this appears to run the first time the code is loaded but not repeated on select box click
        async move {
            let cache = generate_cache(&selected_indexes).await;

            // printing the cache to the console
            console::log_1(&serde_wasm_bindgen::to_value("cache from within the plot_component function").unwrap());
            console::log_1(&serde_wasm_bindgen::to_value(&cache).unwrap());

            let id = "plot-div";
            let mut plot = Plot::new();

            console::log_1(&serde_wasm_bindgen::to_value("cache.energy_values").unwrap());
            console::log_1(&serde_wasm_bindgen::to_value(&cache.energy_values).unwrap());
            for (i, (energy, cross_section)) in cache.energy_values.iter().zip(&cache.cross_section_values).enumerate() {
                if cache.checkbox_selected[i] {
                    let trace = Scatter::new(energy.clone(), cross_section.clone())
                        .name(&format!("Scatter Plot {}", i)
                    );
                    plot.add_trace(trace);
                }
            }

            let layout = plotly::Layout::new()
                .title("Cross sections plotted with XSPlot.com")
                .show_legend(true)
                .x_axis(plotly::layout::Axis::new().title("Energy"))
                .y_axis(plotly::layout::Axis::new().title("Cross section"));
            plot.set_layout(layout);

            plotly::bindings::new_plot(id, &plot).await;
            Ok(())
        }
    });

    // Only on first render
    use_effect_with(selected_indexes.clone(), move |_| {
        p.run();
    });

    html! {
        <div id="plot-div"></div>
    }
}

async fn generate_cache(selected: &HashSet<usize>) -> XsCache {
    // as nothing is selected initially this returns an empy strut
    // I need this calling and updating the cache on every checkbox interaction

    let mut cache_energy_values = Vec::new();
    let mut cache_cross_section_values = Vec::new();
    let mut cache_checkbox_selected = Vec::new();
    console::log_1(&serde_wasm_bindgen::to_value("selected_id").unwrap());
    for &selected_id in selected.iter() {
        let (energy, cross_section) = get_values_by_id(selected_id as i32).await.expect("Failed to get values by ID");
        cache_energy_values.push(energy);
        cache_cross_section_values.push(cross_section);
        cache_checkbox_selected.push(true);

        // Print the selected ID to the console
        
        console::log_1(&selected_id.clone().into());
    }

    // not sure why but this appears to be returning the same sort of data as the below hard coded version but it doesn't plot
    XsCache {
        energy_values: cache_energy_values,
        cross_section_values: cache_cross_section_values,
        checkbox_selected: cache_checkbox_selected,
    }

}

// #[cached(result = true, key = "()", convert = r#"{}"#)]
async fn get_values_by_id(id: i32) -> Result<(Vec<f64>, Vec<f64>), reqwest::Error> {
    let url = format!("https://raw.githubusercontent.com/shimwell/example_yew_rust_table/main/data_{}.json", id);
    let downloaded_reaction_data: ReactionData = reqwest::get(url)
        .await?
        .json()
        .await?;
        console::log_1(&serde_wasm_bindgen::to_value("downloaded data").unwrap());
        console::log_1(&serde_wasm_bindgen::to_value(&downloaded_reaction_data).unwrap());
    Ok((downloaded_reaction_data.energy_values, downloaded_reaction_data.cross_section_values))
}

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

    let sum = selected_indexes.current().len();


    // Column definition
    let columns = vec![
        ColumnBuilder::new("select").orderable(true).short_name("Select").data_property("select").header_class("user-select-none").build(),
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
            checked: selected_indexes.current().contains(&index),
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
        .show_first_last(true)
        .list_classes(vec!(String::from("pagination")))
        .item_classes(vec!(String::from("page-item")))
        .link_classes(vec!(String::from("page-link")))
        .active_item_classes(vec!(String::from("active")))
        .disabled_item_classes(vec!(String::from("disabled")));

    // Handle changing page
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
            <h5>{"Number selected"} <span class="badge text-bg-secondary">{sum}</span></h5>
            <div id="plot-div"></div>
            <App selected_indexes={(*selected_indexes.current()).clone()} />
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
