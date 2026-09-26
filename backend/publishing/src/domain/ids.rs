//! Identifiers of the Publishing context.

xetaravel_kernel::define_id!(
    /// Identifier of an [`super::Article`].
    ArticleId
);
xetaravel_kernel::define_id!(
    /// Identifier of a [`super::Category`].
    CategoryId
);
xetaravel_kernel::define_id!(
    /// Identifier of the author of an article. Publishing only knows authors
    /// by id; their public names come from the `AuthorDirectory` port.
    AuthorId
);
