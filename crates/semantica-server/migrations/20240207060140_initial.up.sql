-- returns current time in utc
CREATE OR REPLACE FUNCTION utc_now() RETURNS TIMESTAMPTZ AS $$
        BEGIN
                RETURN NOW() AT TIME ZONE 'utc';
        END;
$$ LANGUAGE plpgsql;


CREATE TABLE users (
    user_id UUID NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    auth_secret TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    last_login TIMESTAMP NOT NULL,
    in_node UUID NOT NULL,
    god_mode BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX index_users_name ON users(name);
CREATE INDEX index_users_in_node ON users(in_node);


CREATE TABLE spells (
    spell_id UUID NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    emoji TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMP,
    created_by UUID REFERENCES users(user_id)
);

CREATE INDEX index_spells_created_by ON spells(created_by);
CREATE INDEX index_spells_created_at ON spells(created_at);


CREATE TABLE nodes (
    node_id UUID NOT NULL PRIMARY KEY,
    content JSONB NOT NULL,
    parent_id UUID REFERENCES nodes(node_id),
    parent_position INT,
    created_at TIMESTAMP,
    created_by UUID REFERENCES users(user_id),
    created_with UUID REFERENCES spells(spell_id)
);

CREATE INDEX index_nodes_parent_id ON nodes(parent_id);
CREATE INDEX index_nodes_natural_parent ON nodes(parent_id) WHERE parent_position IS NULL;
CREATE INDEX index_nodes_created_at ON nodes(created_at);
CREATE INDEX index_nodes_created_by ON nodes(created_by);

ALTER TABLE users ADD CONSTRAINT fk_in_node FOREIGN KEY(in_node) REFERENCES nodes(node_id);


CREATE TABLE root_nodes (
    node_id UUID NOT NULL PRIMARY KEY REFERENCES nodes(node_id)
);


CREATE TABLE recipes (
    recipe_id UUID NOT NULL PRIMARY KEY,
    product UUID REFERENCES spells(spell_id),
    ingredients UUID[] NOT NULL
);

CREATE INDEX index_recipes_product ON recipes(product);
-- note: we derive the recipe id from the ingredients, so we don't need an index for that.

CREATE TABLE known_recipes (
    recipe_id UUID NOT NULL REFERENCES recipes(recipe_id),
    user_id UUID NOT NULL REFERENCES users(user_id),
    created_at TIMESTAMP NOT NULL,
    UNIQUE (recipe_id, user_id)
);

CREATE INDEX index_known_recipes_recipe_id ON known_recipes(recipe_id);
CREATE INDEX index_known_recipes_user_id ON known_recipes(user_id);
CREATE INDEX index_known_recipes_created_at ON known_recipes(created_at);


CREATE TABLE inventory_contents (
    user_id UUID NOT NULL REFERENCES users(user_id),
    spell_id UUID NOT NULL REFERENCES spells(spell_id),
    amount INT NOT NULL CHECK(amount > 0),
    UNIQUE (user_id, spell_id)
);

CREATE INDEX index_inventory_contents_user_id ON inventory_contents(user_id);
CREATE INDEX index_inventory_contents_user_id_spell_id ON inventory_contents(user_id, spell_id);


CREATE TABLE properties (
    key UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    value JSONB NOT NULL
);

-- whether the game has been initialized
INSERT INTO properties (
    key,
    value
) VALUES (
    '1c02e958-b74c-48f3-97e8-a7d5a8f53703',
    'false'
);
