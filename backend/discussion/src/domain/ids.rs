//! Identifiers of the Discussion context. Articles and authors belong to
//! other contexts: Discussion keeps its own local id types for them and only
//! ever references them by id.

xetaravel_kernel::define_id!(
    /// Identifier of a [`super::Comment`].
    CommentId
);
xetaravel_kernel::define_id!(
    /// Identifier of the commented article (owned by Publishing).
    ArticleId
);
xetaravel_kernel::define_id!(
    /// Identifier of the author of a comment (owned by Identity).
    AuthorId
);
