use itertools::Itertools;

pub type Duration = f64;

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub title: String,
    pub duration: Duration,
}

impl Track {
    pub fn new(title: impl Into<String>, duration: Duration) -> Self {
        Self {
            title: title.into(),
            duration,
        }
    }
}

pub struct TracklistPermutations<'a> {
    inner: Box<dyn Iterator<Item = Vec<&'a Track>> + 'a>,
}

impl<'a> TracklistPermutations<'a> {
    pub fn new(tracks: &'a [Track]) -> Self {
        let len = tracks.len();
        Self {
            inner: Box::new(tracks.iter().permutations(len)),
        }
    }
}

impl<'a> Iterator for TracklistPermutations<'a> {
    type Item = Vec<&'a Track>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools; // for sorted()

    #[test]
    fn test_tracklist_permutations() {
        let tracks = vec![
            Track::new("A", 3.5),
            Track::new("B", 4.0),
            Track::new("C", 2.75),
        ];

        let perms: Vec<_> = TracklistPermutations::new(&tracks)
            .map(|p| p.iter().map(|t| t.title.as_str()).collect::<Vec<_>>())
            .collect();

        // We expect exactly 3! = 6 permutations
        assert_eq!(perms.len(), 6);

        // All permutations should be unique
        let unique_count = perms.iter().sorted().dedup().count();
        assert_eq!(unique_count, perms.len());

        // The expected permutations of titles
        let expected = vec![
            vec!["A", "B", "C"],
            vec!["A", "C", "B"],
            vec!["B", "A", "C"],
            vec!["B", "C", "A"],
            vec!["C", "A", "B"],
            vec!["C", "B", "A"],
        ];

        // Sort both for comparison regardless of order
        let mut perms_sorted = perms.clone();
        perms_sorted.sort();
        let mut expected_sorted = expected.clone();
        expected_sorted.sort();

        assert_eq!(perms_sorted, expected_sorted);
    }
}
