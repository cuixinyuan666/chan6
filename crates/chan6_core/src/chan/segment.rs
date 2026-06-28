use super::model::{ChanBi, ChanDirection, ChanSegment, CHAN_SEGMENT_N_LINE};

pub const DEFAULT_MIN_BI_COUNT_FOR_SEGMENT: usize = 3;

pub fn build_segments(bis: &[ChanBi]) -> Vec<ChanSegment> {
    build_segments_with_min_bi_count(bis, DEFAULT_MIN_BI_COUNT_FOR_SEGMENT)
}

pub fn build_segments_with_min_bi_count(bis: &[ChanBi], min_bi_count: usize) -> Vec<ChanSegment> {
    if bis.len() < min_bi_count || min_bi_count < DEFAULT_MIN_BI_COUNT_FOR_SEGMENT {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut start_index = 0;

    while start_index < bis.len() {
        if start_index + 2 < bis.len() {
            let end_index = latest_segment_end_index_from(bis, start_index);
            let reason = if segments.is_empty() {
                "chanpy_min_three_bi_link"
            } else {
                "chanpy_reversal_three_bi_segment"
            };

            segments.push(make_segment(
                segments.len(),
                &bis[start_index],
                &bis[end_index],
                reason,
            ));
            start_index = end_index + 1;
        } else {
            if segments.len() == 1 {
                let tail = &bis[start_index];
                segments.push(make_segment(
                    segments.len(),
                    tail,
                    tail,
                    "chanpy_even_trailing_bi_segment",
                ));
            }
            break;
        }
    }

    segments
}

fn latest_segment_end_index_from(bis: &[ChanBi], start_index: usize) -> usize {
    debug_assert!(start_index + 2 < bis.len());

    let segment_direction =
        ChanDirection::from_prices(bis[start_index].start_price, bis[start_index + 2].end_price);
    let mut segment_end_index = start_index + 2;
    let mut segment_end_price = bis[segment_end_index].end_price;

    let mut candidate_end_index = segment_end_index + 2;
    while candidate_end_index < bis.len() {
        let candidate = &bis[candidate_end_index];
        if extends_segment(segment_direction, candidate.end_price, segment_end_price) {
            segment_end_index = candidate_end_index;
            segment_end_price = candidate.end_price;
            candidate_end_index += 2;
        } else {
            break;
        }
    }

    segment_end_index
}

fn extends_segment(
    segment_direction: ChanDirection,
    candidate_end_price: f64,
    current_end_price: f64,
) -> bool {
    match segment_direction {
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

    #[test]
    fn failed_extension_can_form_second_reversal_segment() {
        let bis = vec![
            bi(0, 1, 5, 8.0, 14.0),
            bi(1, 5, 9, 14.0, 6.5),
            bi(2, 9, 13, 6.5, 15.0),
            bi(3, 13, 17, 15.0, 5.8),
            bi(4, 17, 21, 5.8, 14.2),
            bi(5, 21, 25, 14.2, 4.0),
            bi(6, 25, 29, 4.0, 13.5),
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
        assert_eq!(segments[1].end_parent_index, Some(5));
        assert_eq!(segments[1].start_bar_id, 13);
        assert_eq!(segments[1].end_bar_id, 25);
        assert_eq!(segments[1].start_price, 15.0);
        assert_eq!(segments[1].end_price, 4.0);
        assert!(!segments[1].confirmed);
    }

    #[test]
    fn failed_extension_can_form_third_reversal_segment() {
        let bis = vec![
            bi(0, 1, 5, 8.0, 14.0),
            bi(1, 5, 9, 14.0, 6.5),
            bi(2, 9, 13, 6.5, 15.0),
            bi(3, 13, 17, 15.0, 5.8),
            bi(4, 17, 21, 5.8, 14.2),
            bi(5, 21, 25, 14.2, 4.0),
            bi(6, 25, 29, 4.0, 13.5),
            bi(7, 29, 33, 13.5, 3.0),
            bi(8, 33, 37, 3.0, 12.5),
            bi(9, 37, 41, 12.5, 2.5),
            bi(10, 41, 45, 2.5, 13.2),
            bi(11, 45, 49, 13.2, 4.2),
            bi(12, 49, 53, 4.2, 16.5),
        ];

        let segments = build_segments(&bis);

        assert_eq!(segments.len(), 3);

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
        assert_eq!(segments[1].end_parent_index, Some(9));
        assert_eq!(segments[1].start_bar_id, 13);
        assert_eq!(segments[1].end_bar_id, 41);
        assert_eq!(segments[1].start_price, 15.0);
        assert_eq!(segments[1].end_price, 2.5);

        assert_eq!(segments[2].index, 2);
        assert_eq!(segments[2].direction, ChanDirection::Up);
        assert_eq!(segments[2].start_parent_index, Some(10));
        assert_eq!(segments[2].end_parent_index, Some(12));
        assert_eq!(segments[2].start_bar_id, 41);
        assert_eq!(segments[2].end_bar_id, 53);
        assert_eq!(segments[2].start_price, 2.5);
        assert_eq!(segments[2].end_price, 16.5);
        assert!(!segments[2].confirmed);
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
