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

impl<T> From<Vec<(T, Duration)>> for Tracklist
where
    T: Into<String>,
{
    fn from(tuples: Vec<(T, Duration)>) -> Self {
        let tracks = tuples
            .into_iter()
            .map(|(title, duration)| Track::new(title, duration))
            .collect();
        Tracklist(tracks)
    }
}

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
        self.inner
            .next()
            .map(|perm| Tracklist::new(perm.into_iter().cloned().collect()))
    }
}

/// Represents a physical medium (e.g. Vinyl, Cassette)
/// with a number of sides and max duration per side.
pub struct Medium {
    pub sides: usize,
    pub max_duration_per_side: Duration,
}

impl Medium {
    pub fn fits(&self, tracklist: &Tracklist) -> bool {
        let durations: Vec<Duration> = tracklist.0.iter().map(|t| t.duration).collect();

        // Reject if the tracklist is longer than the medium's total capacity
        if tracklist.duration() > self.sides as f64 * self.max_duration_per_side {
            return false;
        }

        let mut sides_used = 1;
        let mut current_sum = 0.0;

        for &d in &durations {
            if current_sum + d <= self.max_duration_per_side {
                current_sum += d;
            } else {
                // Need to start a new side
                sides_used += 1;
                if sides_used > self.sides {
                    return false;
                }
                current_sum = d;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    // Implement Ord only for tests, to allow sorting Tracklists by lex order of titles.
    impl PartialOrd for Tracklist {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for Tracklist {
        fn cmp(&self, other: &Self) -> Ordering {
            self.0
                .iter()
                .map(|t| &t.title)
                .cmp(other.0.iter().map(|t| &t.title))
        }
    }

    #[test]
    fn test_tracklist_permutations() {
        let tracks = vec![
            Track::new("A", 3.5),
            Track::new("B", 4.0),
            Track::new("C", 2.75),
        ];

        let mut perms: Vec<_> = TracklistPermutations::new(&tracks).collect();

        assert_eq!(perms.len(), 6);

        let mut expected = vec![
            Tracklist::from(vec![("A", 3.5), ("B", 4.0), ("C", 2.75)]),
            Tracklist::from(vec![("A", 3.5), ("C", 2.75), ("B", 4.0)]),
            Tracklist::from(vec![("B", 4.0), ("A", 3.5), ("C", 2.75)]),
            Tracklist::from(vec![("B", 4.0), ("C", 2.75), ("A", 3.5)]),
            Tracklist::from(vec![("C", 2.75), ("A", 3.5), ("B", 4.0)]),
            Tracklist::from(vec![("C", 2.75), ("B", 4.0), ("A", 3.5)]),
        ];

        perms.sort();
        expected.sort();

        assert_eq!(perms, expected);
    }

    #[test]
    fn test_medium_fits() {
        let tracks = Tracklist::from(vec![("A", 10.0), ("B", 8.0), ("C", 12.0), ("D", 7.0)]);
        let medium = Medium {
            sides: 2,
            max_duration_per_side: 20.0,
        };
        assert!(medium.fits(&tracks)); // e.g. (A+B=18), (C+D=19)

        let medium2 = Medium {
            sides: 2,
            max_duration_per_side: 15.0,
        };
        assert!(!medium2.fits(&tracks)); // no 2-side split possible with max 15

        let medium3 = Medium {
            sides: 3,
            max_duration_per_side: 12.0,
        };
        assert!(!medium3.fits(&tracks)); // no 3-side split possible with max 12

        let tracks2 = Tracklist::from(vec![("A", 10.0), ("B", 5.0), ("C", 7.0), ("D", 7.0)]);
        assert!(medium3.fits(&tracks2)); // (A=10), (B+C=12), (D=7)

        let tracks3 = Tracklist::from(vec![
            ("A", 21.0), // longer than max 20
            ("B", 5.0),
        ]);
        let medium4 = Medium {
            sides: 2,
            max_duration_per_side: 20.0,
        };
        assert!(!medium4.fits(&tracks3));
    }
}
