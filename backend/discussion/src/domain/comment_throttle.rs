use chrono::{DateTime, Duration, Utc};
use xetaravel_kernel::{DomainError, DomainResult};

use super::{AuthorId, Comment};

/// Anti-flood policy applied before a member posts a comment on an article.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentThrottle {
    /// How long a member who wrote the last comment of an article must wait
    /// (unless someone replies) before commenting again.
    pub double_post_window: Duration,
    /// Minimum delay between two comments of the same member on an article.
    pub cooldown: Duration,
}

impl Default for CommentThrottle {
    /// 12 hours between two consecutive comments, 5 minutes between replies.
    fn default() -> Self {
        Self {
            double_post_window: Duration::hours(12),
            cooldown: Duration::minutes(5),
        }
    }
}

impl CommentThrottle {
    /// Fails with `DomainError::TooManyRequests` when `author` may not comment yet.
    ///
    /// `last_on_article` is the latest comment of the article (any author),
    /// `last_by_author` the latest comment of `author` on the same article.
    /// Rules: `author` cannot post twice in a row within `double_post_window`,
    /// and must wait `cooldown` after their own last comment even when someone
    /// replied in the meantime.
    pub fn ensure_can_post(
        &self,
        author: AuthorId,
        last_on_article: Option<&Comment>,
        last_by_author: Option<&Comment>,
        now: DateTime<Utc>,
    ) -> DomainResult<()> {
        if let Some(last) = last_on_article.filter(|last| last.author_id == author) {
            let remaining = last.created_at + self.double_post_window - now;
            if remaining > Duration::zero() {
                return Err(DomainError::TooManyRequests(format!(
                    "you already posted the last comment, wait for a reply or try again in {}",
                    format_remaining(remaining)
                )));
            }
        }

        if let Some(last) = last_by_author {
            let remaining = last.created_at + self.cooldown - now;
            if remaining > Duration::zero() {
                return Err(DomainError::TooManyRequests(format!(
                    "please wait {} before commenting again",
                    format_remaining(remaining)
                )));
            }
        }

        Ok(())
    }
}

/// Formats a strictly positive waiting time for humans, rounded up: in hours
/// above one hour, in minutes otherwise (e.g. "11 hours", "1 minute").
fn format_remaining(remaining: Duration) -> String {
    let (amount, unit) = if remaining > Duration::hours(1) {
        (ceil_div(remaining.num_seconds(), 3600), "hour")
    } else {
        (ceil_div(remaining.num_seconds(), 60).max(1), "minute")
    };
    let plural = if amount > 1 { "s" } else { "" };
    format!("{amount} {unit}{plural}")
}

/// Divides `value` by `divisor`, rounding up (both are positive).
fn ceil_div(value: i64, divisor: i64) -> i64 {
    (value + divisor - 1) / divisor
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::domain::ArticleId;

    /// Returns the instant the tests are frozen at.
    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap()
    }

    /// Builds a comment of `author` written `ago` before [`now`].
    fn comment(author: AuthorId, ago: Duration) -> Comment {
        Comment::post(ArticleId::generate(), author, "Hello", now() - ago).unwrap()
    }

    /// Returns the waiting message of a refused post (panics otherwise).
    fn refusal(result: DomainResult<()>) -> String {
        match result {
            Err(DomainError::TooManyRequests(message)) => message,
            other => panic!("expected a refusal, got {other:?}"),
        }
    }

    #[test]
    fn defaults_to_12_hours_and_5_minutes() {
        let throttle = CommentThrottle::default();
        assert_eq!(throttle.double_post_window, Duration::hours(12));
        assert_eq!(throttle.cooldown, Duration::minutes(5));
    }

    #[test]
    fn accepts_the_first_comment() {
        let throttle = CommentThrottle::default();
        assert!(
            throttle
                .ensure_can_post(AuthorId::generate(), None, None, now())
                .is_ok()
        );
    }

    #[test]
    fn refuses_a_double_post_within_the_window() {
        let author = AuthorId::generate();
        let last = comment(author, Duration::minutes(50));

        let message = refusal(CommentThrottle::default().ensure_can_post(
            author,
            Some(&last),
            Some(&last),
            now(),
        ));

        assert_eq!(
            message,
            "you already posted the last comment, wait for a reply or try again in 12 hours"
        );
    }

    #[test]
    fn double_post_message_rounds_the_remaining_time_up() {
        let author = AuthorId::generate();
        let throttle = CommentThrottle::default();

        let last = comment(author, Duration::minutes(10 * 60 + 30));
        let message = refusal(throttle.ensure_can_post(author, Some(&last), Some(&last), now()));
        assert!(message.ends_with("try again in 2 hours"), "{message}");

        let last = comment(author, Duration::minutes(11 * 60 + 30));
        let message = refusal(throttle.ensure_can_post(author, Some(&last), Some(&last), now()));
        assert!(message.ends_with("try again in 30 minutes"), "{message}");

        let last = comment(author, Duration::hours(12) - Duration::seconds(30));
        let message = refusal(throttle.ensure_can_post(author, Some(&last), Some(&last), now()));
        assert!(message.ends_with("try again in 1 minute"), "{message}");
    }

    #[test]
    fn accepts_a_double_post_once_the_window_is_over() {
        let author = AuthorId::generate();
        let last = comment(author, Duration::hours(12));

        assert!(
            CommentThrottle::default()
                .ensure_can_post(author, Some(&last), Some(&last), now())
                .is_ok()
        );
    }

    #[test]
    fn refuses_a_reply_within_the_cooldown() {
        let author = AuthorId::generate();
        let own = comment(author, Duration::minutes(1));
        let reply = comment(AuthorId::generate(), Duration::seconds(10));

        let message = refusal(CommentThrottle::default().ensure_can_post(
            author,
            Some(&reply),
            Some(&own),
            now(),
        ));

        assert_eq!(message, "please wait 4 minutes before commenting again");
    }

    #[test]
    fn accepts_a_reply_once_the_cooldown_is_over() {
        let author = AuthorId::generate();
        let own = comment(author, Duration::minutes(5));
        let reply = comment(AuthorId::generate(), Duration::minutes(1));

        assert!(
            CommentThrottle::default()
                .ensure_can_post(author, Some(&reply), Some(&own), now())
                .is_ok()
        );
    }

    #[test]
    fn ignores_the_comments_of_other_authors() {
        let other = comment(AuthorId::generate(), Duration::seconds(1));

        assert!(
            CommentThrottle::default()
                .ensure_can_post(AuthorId::generate(), Some(&other), None, now())
                .is_ok()
        );
    }

    #[test]
    fn formats_the_remaining_time() {
        assert_eq!(format_remaining(Duration::seconds(1)), "1 minute");
        assert_eq!(format_remaining(Duration::seconds(61)), "2 minutes");
        assert_eq!(format_remaining(Duration::hours(1)), "60 minutes");
        assert_eq!(format_remaining(Duration::minutes(61)), "2 hours");
    }
}
