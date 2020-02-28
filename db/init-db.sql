CREATE TABLE events (
    aggregateId VARCHAR(40) NOT NULL,
    sequence int NOT NULL,
    time VARCHAR(64) NOT NULL,
    payloadType VARCHAR(128) NOT NULL,
    payload jsonb NOT NULL,
    metadata jsonb NOT NULL,
    PRIMARY KEY(aggregateId, sequence)
);