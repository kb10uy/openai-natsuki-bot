CREATE TABLE conversations(
    id TEXT NOT NULL PRIMARY KEY,
    conversation_blob BLOB NOT NULL
);

CREATE TABLE platform_contexts(
    conversation_id TEXT NOT NULL PRIMARY KEY,
    platform TEXT NOT NULL,
    context TEXT NOT NULL
);
CREATE INDEX platform_context_index ON platform_contexts(platform, context);

CREATE TABLE skeb_illusts(
    url TEXT NOT NULL PRIMARY KEY,
    creator_name TEXT NOT NULL,
    comment TEXT NOT NULL
);
