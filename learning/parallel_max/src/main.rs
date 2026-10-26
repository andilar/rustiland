fn main() {
    let arr = &[1, 25, -4, 10, 1223, 1, 112332, 123];
    let max = find_max(arr).expect("Das Beispielarray ist nicht leer");
    println!("Maximum: {}", max);
}

fn find_max(arr: &[i32]) -> Option<i32> {
    const THRESHOLD: usize = 2;

    if arr.len() <= THRESHOLD {
        return arr.iter().copied().max();
    }

    let mid = arr.len() / 2;
    let (left, right) = arr.split_at(mid);

    crossbeam::scope(|s| {
        let thread_l = s.spawn(|_| find_max(left));
        let thread_r = s.spawn(|_| find_max(right));

        let max_l = thread_l.join().unwrap()?;
        let max_r = thread_r.join().unwrap()?;

        Some(max_l.max(max_r))
    })
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::find_max;

    #[test]
    fn finds_example_maximum() {
        let arr = [1, 25, -4, 10, 1223, 1, 112332, 123];
        assert_eq!(find_max(&arr), Some(112332));
    }

    #[test]
    fn returns_none_for_empty_input() {
        assert_eq!(find_max(&[]), None);
    }

    #[test]
    fn handles_single_values_and_pairs() {
        assert_eq!(find_max(&[-7]), Some(-7));
        assert_eq!(find_max(&[5, -3]), Some(5));
        assert_eq!(find_max(&[-3, 5]), Some(5));
    }

    #[test]
    fn finds_maximum_in_negative_values() {
        assert_eq!(find_max(&[-12, -3, -27, -1, -8]), Some(-1));
    }

    #[test]
    fn handles_odd_lengths_duplicates_and_integer_limits() {
        assert_eq!(
            find_max(&[i32::MIN, i32::MAX, 0, i32::MAX, -1]),
            Some(i32::MAX)
        );
        assert_eq!(find_max(&[i32::MIN; 3]), Some(i32::MIN));
    }
}
