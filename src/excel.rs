use anyhow::Result;
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook};

use crate::config::COINS;
use crate::models::ReportRow;
use crate::totals::Totals;

pub fn generate_excel_report(
    path: &str,
    rows: &[ReportRow],
    totals: &Totals,
    start_date: &Option<String>,
    end_date: &Option<String>,
) -> Result<()> {
    let mut workbook = Workbook::new();

    let worksheet = workbook.add_worksheet();

    worksheet.set_name("PPS Report")?;

    worksheet.set_freeze_panes(1, 0)?;

    // ----------------------------------------
    // FORMATS
    // ----------------------------------------

    let header_format = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::Blue)
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);

    let cell_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::VerticalCenter);

    let number_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_num_format("0.00000000");

    let total_format = Format::new()
        .set_bold()
        .set_background_color(Color::Yellow)
        .set_border(FormatBorder::Thin)
        .set_num_format("0.00000000");

    let grand_total_format = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::Green)
        .set_border(FormatBorder::Thin)
        .set_num_format("0.00000000");

    // ----------------------------------------
    // HEADERS
    // ----------------------------------------

    worksheet.write_with_format(0, 0, "ИМЯ", &header_format)?;
    worksheet.write_with_format(0, 1, "ССЫЛКА", &header_format)?;
    worksheet.write_with_format(0, 2, "МОНЕТА", &header_format)?;
    worksheet.write_with_format(0, 3, "PPS PROFIT", &header_format)?;
    worksheet.write_with_format(0, 4, "ССУММА PPS У ПОЛЬЗОВАТЕЛЯ", &header_format)?;

    // ----------------------------------------
    // WIDTHS
    // ----------------------------------------

    worksheet.set_column_width(0, 25)?;
    worksheet.set_column_width(1, 100)?;
    worksheet.set_column_width(2, 15)?;
    worksheet.set_column_width(3, 20)?;
    worksheet.set_column_width(4, 40)?;

    // ----------------------------------------
    // DATE RANGE
    // ----------------------------------------

    if let (Some(start), Some(end)) = (start_date, end_date) {
        worksheet.write(0, 6, format!("Период: {} -> {}", start, end))?;
    } else {
        worksheet.write(0, 6, "Период: ALL")?;
    }

    // ----------------------------------------
    // DATA
    // ----------------------------------------

    let mut row_index: u32 = 1;

    for row in rows {
        let start_merge_row = row_index;

        for coin in COINS {
            let value = row.coin_values.get(coin).unwrap_or(&0.0);

            worksheet.write_with_format(row_index, 2, coin, &cell_format)?;

            worksheet.write_number_with_format(row_index, 3, *value, &number_format)?;

            row_index += 1;
        }

        let end_merge_row = row_index - 1;

        worksheet.merge_range(
            start_merge_row,
            0,
            end_merge_row,
            0,
            &row.client_name,
            &cell_format,
        )?;

        worksheet.merge_range(
            start_merge_row,
            1,
            end_merge_row,
            1,
            &row.original_url,
            &cell_format,
        )?;

        worksheet.merge_range(
            start_merge_row,
            4,
            end_merge_row,
            4,
            &format!("{:.8}", row.total_per_link),
            &total_format,
        )?;
    }

    // ----------------------------------------
    // TOTALS
    // ----------------------------------------

    row_index += 2;

    worksheet.write_with_format(row_index, 1, "СУММА ПО ВСЕМ МОНЕТАМ", &total_format)?;

    row_index += 1;

    for coin in COINS {
        let value = totals.per_coin.get(coin).unwrap_or(&0.0);

        worksheet.write_with_format(row_index, 2, coin, &total_format)?;

        worksheet.write_number_with_format(row_index, 3, *value, &total_format)?;

        row_index += 1;
    }

    row_index += 1;

    worksheet.write_with_format(row_index, 2, "ОБЩАЯ СУММА", &grand_total_format)?;

    worksheet.write_number_with_format(row_index, 3, totals.grand_total, &grand_total_format)?;

    worksheet.autofilter(0, 0, row_index, 4)?;

    workbook.save(path)?;

    Ok(())
}
