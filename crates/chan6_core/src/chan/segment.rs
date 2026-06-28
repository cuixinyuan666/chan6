use super::model::{ChanBi, ChanDirection, ChanSegment, CHAN_SEGMENT_N_LINE};

pub const DEFAULT_MIN_BI_COUNT_FOR_SEGMENT: usize = 3;

pub fn build_segments(bis: &[ChanBi]) -> Vec<ChanSegment> {
    build_segments_with_min_bi_count(bis, DEFAULT_MIN_BI_COUNT_FOR_SEGMENT)
}

pub fn build_segments_with_min_bi_count(bis: &[ChanBi], min_bi_count: usize) -> Vec<ChanSegment> {
    if bis.len() < min_bi_count || min_bi_count < DEFAULT_MIN_BI_COUNT_FOR_SEGMENT {
        return Vec::new();
    }

    let main_end_index = latest_confirmed_like_main_end_index(bis);
    let mut segments = vec![make_segment(
        0,
        &bis[0],
        &bis[main_end_index],
        "chanpy_min_three_bi_link",
    )];

    if main_end_index + 1 < bis.len() {
        let tail = &bis[main_end_index + 1];
        segments.push(make_segment(
            1,
            tail,
            tail,
            "chanpy_even_trailing_bi_segment",
        ));
    }

    segments
}

fn latest_confirmed_like_main_end_index(bis: &[ChanBi]) -> usize {
    debug_assert!(bis.len() >= DEFAULT_MIN_BI_COUNT_FOR_SEGMENT);

    let main_direction = ChanDirection::from_prices(bis[0].start_price, bis[2].end_price);
    let mut main_end_index = 2;
    let mut main_end_price = bis[2].end_price;

    let mut candidate_end_index = 4;
    while candidate_end_index < bis.len() {
        let candidate = &bis[candidate_end_index];
        if extends_main_segment(main_direction, candidate.end_price, main_end_price) {
            main_end_index = candidate_end_index;
            main_end_price = candidate.end_price;
            candidate_end_index += 2;
        } else {
            break;
        }
    }

    main_end_index
}

fn extends_main_segment(
    main_direction: ChanDirection,
    candidate_end_price: f64,
    current_end_price: f64,
) -> bool {
    match main_direction {
        ChanDirection::Up => candidate_end_price > current_end_price,
        ChanDirection::Down => candidate_end_price < current_end_price,
        ChanDirection::Unknown => false,
    }
}

fn make_segment(index: usize, start: &ChanBi, end: &ChanBi, reason: &str) -> ChanSegment {
    ChanSegment {
        n: CHAN_SEGMENT_N_LINE,
        input_n: None,
        index,
        direction: ChanDirection::from_prices(start.start_price, end.end_price),
        start_parent_index: Some(start.index),
        end_parent_index: Some(end.index),
        start_bar_id: start.start_bar_id,
        start_price: start.start_price,
        end_bar_id: end.end_bar_id,
        end_price: end.end_price,
        confirmed: false,
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::build_segments;
    use crate::chan::model::{ChanBi, ChanDirection, CHAN_SEGMENT_N_LINE};

    #[test]
    fn fewer_than_three_bis_have_no_segment() {
        let bis = vec![bi(0, 1, 5, 8.0, 14.0), bi(1, 5, 9, 14.0, 6.5)];
        let segments = build_segments(&bis);
        assert!(segments.is_empty());
    }

    #[test]
    fn three_bis_form_minimal_unconfirmed_line_segment() {
        let bis = vec![
            bi(0, 1, 5, 8.0, 14.0),
            bi(1, 5, 9, 14.0, 6.5),
            bi(2, 9, 13, 6.5, 15.0),
        ];

        let segments = build_segments(&bis);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].n, CHAN_SEGMENT_N_LINE);
        assert_eq!(segments[0].index, 0);
        assert_eq!(segments[0].direction, ChanDirection::Up);
        assert_eq!(segments[0].start_parent_index, Some(0));
        assert_eq!(segments[0].end_parent_index, Some(2));
        assert_eq!(segments[0].start_bar_id, 1);
        assert_eq!(segments[0].start_price, 8.0);
        assert_eq!(segments[0].end_bar_id, 13);
        assert_eq!(segments[0].end_price, 15.0);
        assert!(!segments[0].confirmed);
        assert_eq!(segments[0].reason, "chanpy_min_three_bi_link");
    }

    #[test]
    fn four_bis_keep_main_odd_segment_and_emit_even_tail_segment() {
        let bis = vec![
            bi(0, 1, 5, 8.0, 14.0),
            bi(1, 5, 9, 14.0, 6.5),
            bi(2, 9, 13, 6.5, 15.0),
            bi(3, 13, 17, 15.0, 5.8),
        ];

        let segments = build_segments(&bis);

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].start_parent_index, Some(0));
        assert_eq!(segments[0].end_parent_index, Some(2));
        assert_eq!(segments[0].direction, ChanDirection::Up);

        assert_eq!(segments[1].index, 1);
        assert_eq!(segments[1].direction, ChanDirection::Down);
        assert_eq!(segments[1].start_parent_index, Some(3));
        assert_eq!(segments[1].end_parent_index, Some(3));
        assert_eq!(segments[1].start_bar_id, 13);
        assert_eq!(segments[1].end_bar_id, 17);
        assert!(!segments[1].confirmed);
        assert_eq!(segments[1].reason, "chanpy_even_trailing_bi_segment");
    }

    #[test]
    fn five_bis_extend_the_unconfirmed_line_segment_to_latest_stronger_endpoint() {
        let bis = vec![
            bi(0, 1, 5, 8.0, 14.0),
            bi(1, 5, 9, 14.0, 6.5),
            bi(2, 9, 13, 6.5, 15.0),
            bi(3, 13, 17, 15.0, 5.8),
            bi(4, 17, 21, 5.8, 16.0),
        ];

        let segments = build_segments(&bis);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].n, CHAN_SEGMENT_N_LINE);
        assert_eq!(segments[0].index, 0);
        assert_eq!(segments[0].direction, ChanDirection::Up);
        assert_eq!(segments[0].start_parent_index, Some(0));
        assert_eq!(segments[0].end_parent_index, Some(4));
        assert_eq!(segments[0].start_bar_id, 1);
        assert_eq!(segments[0].end_bar_id, 21);
        assert_eq!(segments[0].end_price, 16.0);
        assert!(!segments[0].confirmed);
    }

    #[test]
    fn five_bis_with_weaker_endpoint_keeps_main_segment_and_even_tail() {
        let bis = vec![
            bi(0, 1, 5, 8.0, 14.0),
            bi(1, 5, 9, 14.0, 6.5),
            bi(2, 9, 13, 6.5, 15.0),
            bi(3, 13, 17, 15.0, 5.8),
            bi(4, 17, 21, 5.8, 14.2),
        ];

        let segments = build_segments(&bis);

        assert_eq!(segments.len(), 2);

        assert_eq!(segments[0].index, 0);
        assert_eq!(segments[0].direction, ChanDirection::Up);
        assert_eq!(segments[0].start_parent_index, Some(0));
        assert_eq!(segments[0].end_parent_index, Some(2));
        assert_eq!(segments[0].start_bar_id, 1);
        assert_eq!(segments[0].end_bar_id, 13);
        assert_eq!(segments[0].end_price, 15.0);

        assert_eq!(segments[1].index, 1);
        assert_eq!(segments[1].direction, ChanDirection::Down);
        assert_eq!(segments[1].start_parent_index, Some(3));
        assert_eq!(segments[1].end_parent_index, Some(3));
        assert_eq!(segments[1].start_bar_id, 13);
        assert_eq!(segments[1].end_bar_id, 17);
        assert_eq!(segments[1].end_price, 5.8);
    }

    fn bi(
        index: usize,
        start_bar_id: i64,
        end_bar_id: i64,
        start_price: f64,
        end_price: f64,
    ) -> ChanBi {
        let mut item = ChanBi::new(
            index,
            index,
            index + 1,
            start_bar_id,
            start_price,
            end_bar_id,
            end_price,
        );
        item.prev_index = index.checked_sub(1);
        item.next_index = Some(index + 1);
        item
    }
}
