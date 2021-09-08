use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Write,
    sync::Arc,
    time::{Duration, Instant},
};

use serenity::prelude::*;
use serenity::{
    async_trait,
    client::{
        Client,
        Context,
        EventHandler,
        bridge::gateway::{
            GatewayIntents,
            ShardId,
            ShardManager},
    },
    framework::standard::{
        buckets::{LimitedFor, RevertBucket},
        help_commands,
        macros::{check, command, group, help, hook},
        Args,
        CommandGroup,
        CommandOptions,
        CommandResult,
        DispatchError,
        HelpOptions,
        Reason,
        StandardFramework,
    },
    http::Http,
    model::{
        channel::{Channel, Message},
        gateway::Ready,
        id::UserId,
        permissions::Permissions,
    },
    utils::{content_safe, ContentSafeOptions},
    Result,
};