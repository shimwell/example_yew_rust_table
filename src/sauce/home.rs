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
use plotly::layout::{AxisType};
use yew::prelude::*;
use serde::Deserialize;
use crate::types::mock_data::Data;

#[derive(Debug, Serialize, Deserialize)]
struct ReactionData {
    #[serde(rename = "energy")]
    energy_values: Vec<f64>,
    #[serde(rename = "cross section")]
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
    let is_y_log = use_state(|| true);
    let is_x_log = use_state(|| true);

    let p = use_async::<_, _, ()>({
        let selected_indexes = selected_indexes.clone();
        let is_y_log = is_y_log.clone();
        let is_x_log = is_x_log.clone();

        async move {
            let cache = generate_cache(&selected_indexes).await;

            let id = "plot-div";
            let mut plot = Plot::new();

            for (i, (energy, cross_section)) in cache.energy_values.iter().zip(&cache.cross_section_values).enumerate() {
                if cache.checkbox_selected[i] {
                    let trace = Scatter::new(energy.clone(), cross_section.clone())
                        .name(&format!("Scatter Plot {}", i));
                    plot.add_trace(trace);
                }
            }

            let y_axis = plotly::layout::Axis::new()
                .title("Cross section")
                .type_(if *is_y_log { AxisType::Log } else { AxisType::Linear });

            let x_axis = plotly::layout::Axis::new()
                .title("Energy")
                .type_(if *is_x_log { AxisType::Log } else { AxisType::Linear });

            let layout = plotly::Layout::new()
                .title("Cross sections plotted with XSPlot.com")
                .show_legend(true)
                .x_axis(x_axis)
                .y_axis(y_axis);
            
            plot.set_layout(layout);

            plotly::bindings::new_plot(id, &plot).await;
            Ok(())
        }
    });

    use_effect_with((selected_indexes.clone(), is_y_log.clone(), is_x_log.clone()), move |_| {
        p.run();
    });

    let onclick_toggle_y_log = {
        let is_y_log = is_y_log.clone();
        Callback::from(move |_| {
            is_y_log.set(!*is_y_log);
        })
    };

    let onclick_toggle_x_log = {
        let is_x_log = is_x_log.clone();
        Callback::from(move |_| {
            is_x_log.set(!*is_x_log);
        })
    };

    html! {
        <div>
            <button 
                onclick={onclick_toggle_y_log}
                class="btn btn-primary mb-2"
            >
                {if *is_y_log { "Switch Y to Linear Scale" } else { "Switch Y to Log Scale" }}
            </button>
            <button 
                onclick={onclick_toggle_x_log}
                class="btn btn-primary mb-2"
            >
                {if *is_x_log { "Switch X to Linear Scale" } else { "Switch X to Log Scale" }}
            </button>
            <div id="plot-div"></div>
        </div>
    }
}

async fn generate_cache(selected: &HashSet<usize>) -> XsCache {
    let mut cache_energy_values = Vec::new();
    let mut cache_cross_section_values = Vec::new();
    let mut cache_checkbox_selected = Vec::new();
    console::log_1(&serde_wasm_bindgen::to_value("selected_id").unwrap());
    for &selected_id in selected.iter() {
        let (energy, cross_section) = get_values_by_id(selected_id as i32).await.expect("Failed to get values by ID");
        cache_energy_values.push(energy);
        cache_cross_section_values.push(cross_section);
        cache_checkbox_selected.push(true);
        console::log_1(&selected_id.clone().into());
    }

    XsCache {
        energy_values: cache_energy_values,
        cross_section_values: cache_cross_section_values,
        checkbox_selected: cache_checkbox_selected,
    }
}

async fn get_values_by_id(id: i32) -> Result<(Vec<f64>, Vec<f64>), reqwest::Error> {
    let data = crate::types::mock_data::Data::default();
    let Some(name) = get_name_by_id(&data, id) else { todo!() };
    let output = convert_string(name);
    console::log_1(&serde_wasm_bindgen::to_value(&"output").unwrap());
    console::log_1(&serde_wasm_bindgen::to_value(&output).unwrap());

    let url = format!("https://raw.githubusercontent.com/openmc-data-storage/ENDF-B-VIII.0-NNDC-json/refs/heads/main/json_files/{output}.json");

    console::log_1(&serde_wasm_bindgen::to_value(&url).unwrap());
    let downloaded_reaction_data: ReactionData = reqwest::get(url)
        .await?
        .json()
        .await?;
        console::log_1(&serde_wasm_bindgen::to_value("downloaded data").unwrap());
        console::log_1(&serde_wasm_bindgen::to_value(&downloaded_reaction_data).unwrap());
    Ok((downloaded_reaction_data.energy_values, downloaded_reaction_data.cross_section_values))
}

fn get_name_by_id(data: &Data, id: i32) -> Option<&String> {
    console::log_1(&serde_wasm_bindgen::to_value("get_name_by_id").unwrap());
    console::log_1(&serde_wasm_bindgen::to_value(&id).unwrap());
    let name = data.data.iter().find(|&&(i, _, _)| i == id).map(|&(_, ref name, _)| name);
    if let Some(name) = name {
        console::log_1(&serde_wasm_bindgen::to_value(&format!("Found name: {}", name)).unwrap());
    } else {
        console::log_1(&serde_wasm_bindgen::to_value("Name not found").unwrap());
    }
    name
}

fn convert_string(input: &str) -> String {
    let mut result = input.to_string();
    result = result.replace("damage-energy", "");
    result = result.replace("heating", "");
    let first_token = result.split_whitespace().next().unwrap_or("");
    let mut letters = String::new();
    let mut numbers = String::new();
    for c in first_token.chars() {
        if c.is_alphabetic() {
            letters.push(c);
        } else if c.is_numeric() {
            numbers.push(c);
        }
    }
    let formatted_first_token = format!("{}_{}", letters, numbers);
    result = result.replacen(first_token, &formatted_first_token, 1);
    while let Some(start) = result.find('(') {
        if let Some(end) = result[start..].find(')') {
            result.replace_range(start..=end + start, "");
        } else {
            break;
        }
    }
    result = result.replace(" MT", "n_");
    result = result.replace(" ", "_");
    result
}

#[function_component(Home)]
pub fn home() -> Html {
    let data = use_reducer(crate::types::mock_data::Data::default);
    let mock_data = (*data).clone();

    let name_search_term = use_state(|| None::<String>);
    let value_search_term = use_state(|| None::<String>);
    let name_search = (*name_search_term).as_ref().cloned();
    let value_search = (*value_search_term).as_ref().cloned();

    let page = use_state(|| 0usize);
    let current_page = (*page).clone();

    let selected_indexes = use_set(HashSet::<usize>::new());
    let sum = selected_indexes.current().len();

    let columns = vec![
        ColumnBuilder::new("select").orderable(true).short_name("Select").data_property("select").header_class("user-select-none").build(),
        ColumnBuilder::new("id").orderable(true).short_name("ID").data_property("id").header_class("user-select-none").build(),
        ColumnBuilder::new("name").orderable(true).short_name("Name").data_property("name").header_class("user-select-none").build(),
        ColumnBuilder::new("value").orderable(true).short_name("Value").data_property("value").header_class("user-select-none").build(),
    ];

    let options = Options {
        unordered_class: Some("fa-sort".to_string()),
        ascending_class: Some("fa-sort-up".to_string()),
        descending_class: Some("fa-sort-down".to_string()),
        orderable_classes: vec!["mx-1".to_string(), "fa-solid".to_string()],
    };

    let callback_sum = {
        let selected_indexes = selected_indexes.clone();
        Callback::from(move |index: usize| {
            if !selected_indexes.insert(index) {
                selected_indexes.remove(&index);
            }
        })
    };

    let filtered_data: Vec<TableLine> = mock_data.data
        .iter()
        .enumerate()
        .filter(|(_, (_, name, value))| {
            let name_match = match name_search {
                Some(ref term) => name.to_lowercase().contains(&term.to_lowercase()),
                None => true,
            };
            let value_match = match value_search {
                Some(ref term) => value.to_string().contains(&*term),
                // Some(ref term) => value.to_string().contains(&term),
                None => true,
            };
            name_match && value_match
        })
        .map(|(index, (id, name, value))| TableLine {
            original_index: index,
            id: *id,
            name: name.clone(),
            value: *value,
            checked: selected_indexes.current().contains(&index),
            sum_callback: callback_sum.clone(),
        })
        .collect();

    let limit = 10;
    let current_page = if filtered_data.is_empty() {
        0
    } else {
        current_page.min((filtered_data.len() - 1) / limit)
    };

    let start_index = current_page * limit;
    let end_index = (start_index + limit).min(filtered_data.len());

    let paginated_data = if filtered_data.is_empty() {
        Vec::new()
    } else {
        filtered_data[start_index..end_index].to_vec()
    };

    let total = filtered_data.len().max(1);

    let oninput_name_search = {
        let name_search_term = name_search_term.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if input.value().is_empty() {
                name_search_term.set(None);
            } else {
                name_search_term.set(Some(input.value()));
            }
        })
    };

    let oninput_value_search = {
        let value_search_term = value_search_term.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if input.value().is_empty() {
                value_search_term.set(None);
            } else {
                value_search_term.set(Some(input.value()));
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

    let handle_page = {
        let page = page.clone();
        Callback::from(move |new_page: usize| {
            page.set(new_page);
        })
    };

    html!(
        <>
            <h1>{"Minimal table Example"}</h1>
            <div class="flex-grow-1 p-2 input-group mb-2">
                <span class="input-group-text">
                    <i class="fas fa-search"></i>
                </span>
                <input 
                    class="form-control" 
                    type="text" 
                    id="name-search" 
                    placeholder="Search by Name" 
                    oninput={oninput_name_search} 
                />
            </div>
            <div class="flex-grow-1 p-2 input-group mb-2">
                <span class="input-group-text">
                    <i class="fas fa-search"></i>
                </span>
                <input 
                    class="form-control" 
                    type="text" 
                    id="value-search" 
                    placeholder="Search by Value" 
                    oninput={oninput_value_search} 
                />
            </div>
            <Table<TableLine> 
                options={options.clone()} 
                limit={Some(limit)} 
                page={current_page} 
                search={name_search.clone()} 
                classes={classes!("table", "table-hover")} 
                columns={columns.clone()} 
                data={paginated_data} 
                orderable={true}
            />
            <Pagination 
                total={total}
                limit={limit} 
                max_pages={6} 
                options={pagination_options} 
                on_page={Some(handle_page)}
            />
            <h5>{"Number selected"} <span class="badge text-bg-secondary">{sum}</span></h5>
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
            "select" => Ok(html!( <input type="checkbox" style="width: 30px; height: 30px;" checked={self.checked}
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