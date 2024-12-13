//! ## Task Description
//!
//! The goal is to develop a backend service for shortening URLs using CQRS
//! (Command Query Responsibility Segregation) and ES (Event Sourcing)
//! approaches. The service should support the following features:
//!
//! ## Functional Requirements
//!
//! ### Creating a short link with a random slug
//!
//! The user sends a long URL, and the service returns a shortened URL with a
//! random slug.
//!
//! ### Creating a short link with a predefined slug
//!
//! The user sends a long URL along with a predefined slug, and the service
//! checks if the slug is unique. If it is unique, the service creates the short
//! link.
//!
//! ### Counting the number of redirects for the link
//!
//! - Every time a user accesses the short link, the click count should
//!   increment.
//! - The click count can be retrieved via an API.
//!
//! ### CQRS+ES Architecture
//!
//! CQRS: Commands (creating links, updating click count) are separated from
//! queries (retrieving link information).
//!
//! Event Sourcing: All state changes (link creation, click count update) must be
//! recorded as events, which can be replayed to reconstruct the system's state.
//!
//! ### Technical Requirements
//!
//! - The service must be built using CQRS and Event Sourcing approaches.
//! - The service must be possible to run in Rust Playground (so no database like
//!   Postgres is allowed)
//! - Public API already written for this task must not be changed (any change to
//!   the public API items must be considered as breaking change).
//! - Event Sourcing should be actively utilized for implementing logic, rather
//!   than existing without a clear purpose.

#![allow(unused_variables, dead_code)]

use rand::{distributions::Alphanumeric, Rng};
use std::collections::{BTreeMap, HashMap};
use url::Url as Vurl;

/// All possible errors of the [`UrlShortenerService`].
#[derive(Debug, PartialEq)]
pub enum ShortenerError {
    /// This error occurs when an invalid [`Url`] is provided for shortening.
    InvalidUrl,

    /// This error occurs when an attempt is made to use a slug (custom alias)
    /// that already exists.
    SlugAlreadyInUse,

    /// This error occurs when the provided [`Slug`] does not map to any existing
    /// short link.
    SlugNotFound,
}

/// A unique string (or alias) that represents the shortened version of the
/// URL.
#[derive(Hash, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
pub struct Slug(pub String);

/// The original URL that the short link points to.
#[derive(Hash, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
pub struct Url(pub String);

/// Shortened URL representation.
#[derive(Hash, Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
pub struct ShortLink {
    /// A unique string (or alias) that represents the shortened version of the
    /// URL.
    pub slug: Slug,

    /// The original URL that the short link points to.
    pub url: Url,
}

/// Statistics of the [`ShortLink`].
#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    /// [`ShortLink`] to which this [`Stats`] are related.
    pub link: ShortLink,

    /// Count of redirects of the [`ShortLink`].
    pub redirects: u64,
}

/// Commands for CQRS.
pub mod commands {
    use super::{ShortLink, ShortenerError, Slug, Url};

    /// Trait for command handlers.
    pub trait CommandHandler {
        /// Creates a new short link. It accepts the original url and an
        /// optional [`Slug`]. If a [`Slug`] is not provided, the service will generate
        /// one. Returns the newly created [`ShortLink`].
        ///
        /// ## Errors
        ///
        /// See [`ShortenerError`].
        fn handle_create_short_link(
            &mut self,
            url: Url,
            slug: Option<Slug>,
        ) -> Result<ShortLink, ShortenerError>;

        /// Processes a redirection by [`Slug`], returning the associated
        /// [`ShortLink`] or a [`ShortenerError`].
        fn handle_redirect(&mut self, slug: Slug) -> Result<ShortLink, ShortenerError>;
    }
}

/// Queries for CQRS
pub mod queries {
    use super::{ShortenerError, Slug, Stats};

    /// Trait for query handlers.
    pub trait QueryHandler {
        /// Returns the [`Stats`] for a specific [`ShortLink`], such as the
        /// number of redirects (clicks).
        ///
        /// [`ShortLink`]: super::ShortLink
        fn get_stats(&self, slug: Slug) -> Result<Stats, ShortenerError>;
    }
}

/// CQRS and Event Sourcing-based service implementation
pub struct UrlShortenerService {
    // TODO: add needed fields
    slugs: HashMap<String, String>,
    stats: BTreeMap<String, u64>,
}

impl UrlShortenerService {
    /// Creates a new instance of the service
    pub fn new() -> Self {
        Self {
            slugs: HashMap::new(),
            stats: BTreeMap::new(),
        }
    }
}

impl commands::CommandHandler for UrlShortenerService {
    fn handle_create_short_link(
        &mut self,
        url: Url,
        slug: Option<Slug>,
    ) -> Result<ShortLink, ShortenerError> {
        let sl: Slug;
        match slug {
            Some(s) => sl = s,
            None => {
                let rnd7: String = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(7)
                    .map(char::from)
                    .collect();
                sl = Slug(rnd7);
            }
        };
        match Vurl::parse(&url.0) {
            Ok(_) => match self.slugs.get(&sl.0) {
                Some(_) => Err(ShortenerError::SlugAlreadyInUse),
                None => {
                    self.slugs.insert(sl.0.clone(), url.0.clone());
                    Ok(ShortLink { url, slug: sl })
                }
            },
            Err(_) => Err(ShortenerError::InvalidUrl),
        }
    }

    fn handle_redirect(&mut self, slug: Slug) -> Result<ShortLink, ShortenerError> {
        match self.slugs.get(&slug.0) {
            Some(u) => {
                let inc = self.stats.entry(slug.0.clone()).or_insert(0);
                *inc += 1;
                Ok(ShortLink {
                    url: Url(u.clone()),
                    slug,
                })
            }
            None => Err(ShortenerError::SlugNotFound),
        }
    }
}

impl queries::QueryHandler for UrlShortenerService {
    fn get_stats(&self, slug: Slug) -> Result<Stats, ShortenerError> {
        todo!()
        // match self.stats.get(&slug.0) {
        //     Some(&redirects) => Ok(Stats {
        //         link: Url(u.clone()),
        //         redirects,
        //     }),
        //     None => Err(ShortenerError::SlugNotFound),
        // }
    }
}

// Dummy fun
fn main() {}

#[cfg(test)]
mod tests {
    use super::{ShortLink, Slug, Url};
    use crate::commands::CommandHandler;
    use crate::UrlShortenerService;

    #[test]
    fn check_create_defined_slug() {
        let my_url = Url("https://www.example.com/".to_string());
        let my_slug = Slug("my-awesome-slug".to_string());
        let mut shortener = UrlShortenerService::new();

        assert_eq!(
            ShortLink {
                url: my_url.clone(),
                slug: my_slug.clone()
            },
            shortener
                .handle_create_short_link(my_url, Some(my_slug))
                .expect("not implemented")
        );
    }

    #[test]
    fn check_one_redirect_defined_slug() {
        let my_url = Url("https://www.example.com/".to_string());
        let my_slug = Slug("my-awesome-slug".to_string());
        let mut shortener = UrlShortenerService::new();

        shortener
            .handle_create_short_link(my_url.clone(), Some(my_slug.clone()))
            .expect("not implemented");

        assert_eq!(
            ShortLink {
                url: my_url,
                slug: my_slug.clone()
            },
            shortener.handle_redirect(my_slug).expect("not implemented")
        );
    }

    #[test]
    fn check_few_redirects_defined_slug() {
        let my_url = Url("https://www.example.com/".to_string());
        let my_slug = Slug("my-awesome-slug".to_string());
        let mut shortener = UrlShortenerService::new();

        shortener
            .handle_create_short_link(my_url.clone(), Some(my_slug.clone()))
            .expect("not implemented");

        for i in 0..10 {
            shortener
                .handle_redirect(my_slug.clone())
                .expect("not implemented");
        }

        assert_eq!(
            ShortLink {
                url: my_url,
                slug: my_slug.clone()
            },
            shortener.handle_redirect(my_slug).expect("not implemented")
        );
    }

    #[test]
    fn check_one_redirect_random_slug() {
        let my_url = Url("https://www.example.com/".to_string());
        let mut shortener = UrlShortenerService::new();

        let short_link = shortener
            .handle_create_short_link(my_url.clone(), None)
            .expect("not implemented");
        let slug = short_link.clone().slug;

        assert_eq!(
            short_link,
            shortener.handle_redirect(slug).expect("not implemented")
        );
    }
}
