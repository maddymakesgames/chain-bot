/*

   All the functions to interact with the database do not return Results as of writing this
   This is to because database errors currently are unrecoverable
   This will be revamped in the future to actually allow error handling instead of panicking

   TODO: Change return types to Results
*/
pub mod guilds;
pub mod leaderboards;
pub mod users;
