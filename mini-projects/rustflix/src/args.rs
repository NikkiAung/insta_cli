use clap::{
    Args,
    Parser,
    Subcommand
};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct RustflixArgs {
    #[clap(subcommand)]
    pub entity_type: EntityType,
}

#[derive(Debug, Subcommand)]
pub enum EntityType {
    User(UserCommand),
    Video(VideoCommand),
    View(ViewCommand)
}

#[derive(Debug, Args)]
pub struct UserCommand {
    #[clap(subcommand)]
    pub command: UserSubCommand,
}

#[derive(Debug, Subcommand)]
pub enum UserSubCommand {
    /// Create a new user
    Create(CreateUser),

    /// Update an existing user
    Update(UpdateUser),

    /// Delete a user
    Delete(DeleteEntity),

    /// Show all users
    Show
}

#[derive(Debug, Args)]
pub struct CreateUser {
    /// The name of the user
    pub name: String,

    /// The email of the user
    pub email: String,
}

#[derive(Debug, Args)]
pub struct UpdateUser {
    /// The ID of the user to update
    pub id: i32,

    /// The new name of the user
    pub name: String,

    /// The new email of the user
    pub email: String,
}

#[derive(Debug, Args)]
pub struct DeleteEntity {
    /// The ID of the entity to delete
    pub id: i32,
}

#[derive(Debug, Args)]
pub struct VideoCommand {
    #[clap(subcommand)]
    pub command: VideoSubCommand,
}

#[derive(Debug, Subcommand)]
pub enum VideoSubCommand {
    /// Create a new video
    Create(CreateVideo),

    /// Update an existing video
    Update(UpdateVideo),

    /// Delete a video
    Delete(DeleteEntity),

    /// Show all videos
    Show
}

#[derive(Debug, Args)]
pub struct CreateVideo {
    /// The title of the video
    pub title: String,

    /// The description of the video
    pub description: String,
}

#[derive(Debug, Args)]
pub struct UpdateVideo {
    /// The id of the video to update
    pub id: i32,

    /// The title of the video
    pub title: String,

    /// The description of the video
    pub description: String,
}

#[derive(Debug, Args)]
pub struct ViewCommand {
    #[clap(subcommand)]
    pub command: ViewSubCommand,
}

#[derive(Debug, Subcommand)]
pub enum ViewSubCommand {
    /// Record a view
    Create(CreateView),

    /// Show all views
    Show,

    /// Show all views with names for users and videos
    ShowPretty,
}

#[derive(Debug, Args)]
pub struct CreateView {
    /// The id of the user who watched the video
    pub user_id: i32,

    /// The id of the video the user watched
    pub video_id: i32,

    /// The time the user watched the video
    pub watch_start: chrono::NaiveDateTime,

    /// How long the user watched the video for
    pub duration: i32,
}
