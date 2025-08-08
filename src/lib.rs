// src/lib.rs
use itertools::Itertools; // bring permutations() into scope

/// Duration type (seconds, using f64)
pub type Duration = f64;

#[derive(Debug, Clone)]
pub struct Track {
    pub title: String,
    pub duration: Duration,
}

impl Track {
    pub fn new<T: Into<String>>(title: T, duration: Duration) -> Self {
        Self {
            title: title.into(),
            duration,
        }
    }
}

/// A Tracklist wrapper (ordered list of tracks).
#[derive(Debug, Clone)]
pub struct Tracklist(pub Vec<Track>);

impl Tracklist {
    pub fn new(tracks: Vec<Track>) -> Self {
        Self(tracks)
    }

    /// Convenience: return titles as Vec<&str>
    pub fn titles(&self) -> Vec<&str> {
        self.0.iter().map(|t| t.title.as_str()).collect()
    }

    /// Total duration
    pub fn duration(&self) -> Duration {
        self.0.iter().map(|t| t.duration).sum()
    }
}

/// Equality for Tracklist: compare the ordered sequence of titles only.
/// This avoids `f64` equality/Eq/Hash problems while still being correct for
/// permutation-checking in tests. If you want durations to be part of equality,
/// switch to an integer duration type (e.g. milliseconds).
impl PartialEq for Tracklist {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        self.0
            .iter()
            .zip(other.0.iter())
            .all(|(a, b)| a.title == b.title)
    }
}
impl Eq for Tracklist {} // safe because we used String equality (total order)

/// Iterator wrapper producing Tracklist permutations lazily.
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
    type Item = Tracklist;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|perm| {
            // clone Tracks into a concrete Tracklist (owned)
            Tracklist::new(perm.into_iter().cloned().collect())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracklist_permutations() {
        let tracks = vec![
            Track::new("A", 3.5),
            Track::new("B", 4.0),
            Track::new("C", 2.75),
        ];

        let perms: Vec<_> = TracklistPermutations::new(&tracks).collect();

        // Expect 3! = 6 permutations
        assert_eq!(perms.len(), 6);

        // Define expected permutations explicitly
        let expected = vec![
            Tracklist::new(vec![
                Track::new("A", 3.5),
                Track::new("B", 4.0),
                Track::new("C", 2.75),
            ]),
            Tracklist::new(vec![
                Track::new("A", 3.5),
                Track::new("C", 2.75),
                Track::new("B", 4.0),
            ]),
            Tracklist::new(vec![
                Track::new("B", 4.0),
                Track::new("A", 3.5),
                Track::new("C", 2.75),
            ]),
            Tracklist::new(vec![
                Track::new("B", 4.0),
                Track::new("C", 2.75),
                Track::new("A", 3.5),
            ]),
            Tracklist::new(vec![
                Track::new("C", 2.75),
                Track::new("A", 3.5),
                Track::new("B", 4.0),
            ]),
            Tracklist::new(vec![
                Track::new("C", 2.75),
                Track::new("B", 4.0),
                Track::new("A", 3.5),
            ]),
        ];

        // Sort both lists to ignore order differences
        let mut perms_sorted = perms.clone();
        perms_sorted.sort_by(|a, b| a.titles().cmp(&b.titles()));
        let mut expected_sorted = expected.clone();
        expected_sorted.sort_by(|a, b| a.titles().cmp(&b.titles()));

        assert_eq!(perms_sorted, expected_sorted);
    }
}
