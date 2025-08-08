use itertools::{Itertools, Permutations}; // for permutations()

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

/// Equality compares only the ordered titles to avoid f64 Eq issues.
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
impl Eq for Tracklist {}

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

/// Iterator producing permutations of a tracklist lazily.
pub struct TracklistPermutations<'a> {
    inner: Permutations<std::slice::Iter<'a, Track>>,
}

impl<'a> TracklistPermutations<'a> {
    pub fn new(tracks: &'a [Track]) -> Self {
        let len = tracks.len();
        Self {
            inner: tracks.iter().permutations(len),
        }
    }
}

impl<'a> Iterator for TracklistPermutations<'a> {
    type Item = Vec<&'a Track>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// Physical medium with sides and max duration per side.
pub struct Medium {
    pub sides: usize,
    pub max_duration_per_side: Duration,
}

impl Medium {
    /// Check if tracklist fits medium sides without splitting tracks.
    pub fn fits(&self, tracklist: &Tracklist) -> bool {
        let durations: Vec<Duration> = tracklist.0.iter().map(|t| t.duration).collect();

        if tracklist.duration() > self.sides as f64 * self.max_duration_per_side {
            return false;
        }

        let mut sides_used = 1;
        let mut current_sum = 0.0;

        for &d in &durations {
            if d > self.max_duration_per_side {
                return false; // track too long for a side
            }
            if current_sum + d <= self.max_duration_per_side {
                current_sum += d;
            } else {
                sides_used += 1;
                if sides_used > self.sides {
                    return false;
                }
                current_sum = d;
            }
        }
        true
    }

    /// Returns true if the two tracks are on the same side when split by duration.
    pub fn on_same_side(&self, tracklist: &Tracklist, t1: &str, t2: &str) -> bool {
        let mut sides_used = 0;
        let mut current_sum = 0.0;
        let mut side_indices = Vec::with_capacity(tracklist.0.len());

        for track in &tracklist.0 {
            if current_sum + track.duration <= self.max_duration_per_side {
                current_sum += track.duration;
            } else {
                sides_used += 1;
                current_sum = track.duration;
            }
            side_indices.push(sides_used);
        }

        let pos1 = tracklist.0.iter().position(|t| t.title == t1);
        let pos2 = tracklist.0.iter().position(|t| t.title == t2);

        if let (Some(i1), Some(i2)) = (pos1, pos2) {
            side_indices[i1] == side_indices[i2]
        } else {
            false // one or both tracks not found
        }
    }
}

/// Kind of constraint (without weight).
#[derive(Debug, Clone)]
pub enum ConstraintKind {
    AtPosition(String, usize),  // (track title, position)
    Adjacent(String, String),   // (track1, track2)
    OnSameSide(String, String), // (track1, track2)
}

/// Constraint with explicit weight.
#[derive(Debug, Clone)]
pub struct Constraint {
    pub kind: ConstraintKind,
    pub weight: usize,
}

/// Score the tracklist against constraints and medium.
pub fn score_tracklist(
    tracklist: &Tracklist,
    constraints: &[Constraint],
    medium: &Medium,
) -> usize {
    let mut score = 0;

    for constraint in constraints {
        match &constraint.kind {
            ConstraintKind::AtPosition(title, pos) => {
                if let Some(track) = tracklist.0.get(*pos) {
                    if &track.title == title {
                        score += constraint.weight;
                    }
                }
            }
            ConstraintKind::Adjacent(t1, t2) => {
                if tracklist
                    .0
                    .windows(2)
                    .any(|w| w[0].title == *t1 && w[1].title == *t2)
                {
                    score += constraint.weight;
                }
            }
            ConstraintKind::OnSameSide(t1, t2) => {
                if medium.on_same_side(tracklist, t1, t2) {
                    score += constraint.weight;
                }
            }
        }
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    // For testing: lex order on track titles.
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

        let mut perms: Vec<Tracklist> = TracklistPermutations::new(&tracks)
            .map(|perm| Tracklist::new(perm.into_iter().cloned().collect()))
            .collect();

        assert_eq!(perms.len(), 6);

        let mut expected: Vec<Tracklist> = vec![
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
    fn test_medium_fits_and_same_side() {
        let tracks = Tracklist::from(vec![("A", 10.0), ("B", 8.0), ("C", 12.0), ("D", 7.0)]);
        let medium = Medium {
            sides: 2,
            max_duration_per_side: 20.0,
        };
        assert!(medium.fits(&tracks)); // (A+B=18), (C+D=19)
        assert!(medium.on_same_side(&tracks, "A", "B"));
        assert!(medium.on_same_side(&tracks, "C", "D"));
        assert!(!medium.on_same_side(&tracks, "B", "C"));
        assert!(!medium.on_same_side(&tracks, "A", "D"));

        let medium2 = Medium {
            sides: 2,
            max_duration_per_side: 15.0,
        };
        assert!(!medium2.fits(&tracks)); // no 2-side split possible with max 15

        let tracks2 = Tracklist::from(vec![("A", 10.0), ("B", 5.0), ("C", 7.0), ("D", 7.0)]);
        let medium3 = Medium {
            sides: 3,
            max_duration_per_side: 12.0,
        };
        assert!(medium3.fits(&tracks2)); // (A=10), (B+C=12), (D=7)
        assert!(medium3.on_same_side(&tracks2, "B", "C"));
        assert!(!medium3.on_same_side(&tracks2, "A", "B"));

        let tracks3 = Tracklist::from(vec![("A", 21.0), ("B", 5.0)]);
        let medium4 = Medium {
            sides: 2,
            max_duration_per_side: 20.0,
        };
        assert!(!medium4.fits(&tracks3));
    }

    #[test]
    fn test_score_tracklist() {
        let medium = Medium {
            sides: 2,
            max_duration_per_side: 10.0,
        };

        let constraints = vec![
            Constraint {
                kind: ConstraintKind::AtPosition("Intro".into(), 0),
                weight: 7,
            },
            Constraint {
                kind: ConstraintKind::Adjacent("First".into(), "Second".into()),
                weight: 5,
            },
            Constraint {
                kind: ConstraintKind::OnSameSide("Second".into(), "Third".into()),
                weight: 2,
            },
        ];

        let max_score = constraints.iter().map(|c| c.weight).sum();

        let tracks = Tracklist::from(vec![
            ("Intro", 5.0),
            ("First", 5.0),
            ("Second", 2.0),
            ("Third", 2.0),
        ]);
        // Tracklist respects all constraints
        assert_eq!(score_tracklist(&tracks, &constraints, &medium), max_score);

        // Third is alone on side 2
        let tracks = Tracklist::from(vec![
            ("Intro", 5.0),
            ("First", 3.0),
            ("Second", 2.0),
            ("Third", 2.0),
        ]);
        assert_eq!(
            score_tracklist(&tracks, &constraints, &medium),
            max_score - constraints[2].weight
        );

        // Third is alone on side 2 and first and second dont folow
        let tracks = Tracklist::from(vec![
            ("Intro", 5.0),
            ("Second", 2.0),
            ("First", 3.0),
            ("Third", 2.0),
        ]);
        assert_eq!(
            score_tracklist(&tracks, &constraints, &medium),
            max_score - constraints[2].weight - constraints[1].weight
        );

        // Third is alone on side 2, first and second dont folow and intro is not at the beginning
        let tracks = Tracklist::from(vec![
            ("Second", 2.0),
            ("Intro", 5.0),
            ("First", 3.0),
            ("Third", 2.0),
        ]);
        assert_eq!(
            score_tracklist(&tracks, &constraints, &medium),
            max_score - constraints[2].weight - constraints[1].weight - constraints[0].weight
        );
    }
}
