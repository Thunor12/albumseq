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

    #[test]
    fn test_tracklist_permutations() {
        let tracks = vec![
            Track::new("A", 3.5),
            Track::new("B", 4.0),
            Track::new("C", 2.75),
        ];

        let perms: Vec<_> = TracklistPermutations::new(&tracks).collect();
        assert_eq!(perms.len(), 6);
        assert_eq!(perms[0][0].title, "A");
        assert_eq!(perms[0][1].title, "B");
        assert_eq!(perms[0][2].title, "C");
    }
}
