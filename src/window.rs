use crate::xylok_stig::*;

use iced::{
    Element,
    Length::{Fill, FillPortion},
    widget::{Column, column, container, row, scrollable, text},
};

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Default)]
pub struct State {
    stig_list: Vec<Stig>,
    current_displayed: Stig,
}

pub fn new() -> State {
    let current_displayed = load_stig("test.txt").unwrap();

    let mut stig_list = Vec::new();

    for _ in 0..100 {
        let mut stig = Stig::default();
        stig.version = String::from("CASA-FW-000260");

        stig_list.push(stig);
    }

    return State {
        stig_list: stig_list,
        current_displayed: current_displayed,
    };
}

pub fn update(state: &mut State, message: Message) {
    match message {}
}

pub fn view(state: &State) -> Element<'_, Message> {
    let containers_vec: Vec<Element<'_, Message>> = state
        .stig_list
        .iter()
        .map(|stig| {
            container(text(stig.version.clone()))
                .padding(10)
                .style(container::rounded_box)
                .width(Fill)
                .into()
        })
        .collect();

    let mut stig_list_col = Column::from_vec(containers_vec);

    let mut current_dislayed_container = container(
        scrollable(column![
            text(state.current_displayed.version.clone()),
            text(state.current_displayed.introduction.clone()),
            text(state.current_displayed.description.clone()),
            text(state.current_displayed.check_text.clone()),
            text(state.current_displayed.fix_text.clone()),
        ])
        .width(Fill),
    )
    .padding(10)
    .style(container::rounded_box);

    stig_list_col = stig_list_col.width(FillPortion(1));
    current_dislayed_container = current_dislayed_container.width(FillPortion(5));

    let final_row = row![scrollable(stig_list_col), current_dislayed_container].height(Fill);

    return final_row.into();
}
