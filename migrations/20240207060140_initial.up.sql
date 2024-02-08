-- returns current time in utc
CREATE OR REPLACE FUNCTION utc_now() RETURNS TIMESTAMPTZ AS $$
        BEGIN
                RETURN NOW() AT TIME ZONE 'utc';
        END;
$$ LANGUAGE plpgsql;


CREATE TABLE users (
    user_id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    auth_secret TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT utc_now(),
    last_login TIMESTAMP NOT NULL DEFAULT utc_now()
);

CREATE INDEX index_users_name ON users(name);


CREATE TABLE spells (
    spell_id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    emoji TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT utc_now(),
    created_by UUID NOT NULL,
    CONSTRAINT fk_created_by FOREIGN KEY(created_by) REFERENCES users(user_id)
);

CREATE INDEX index_spells_created_by ON spells(created_by);
CREATE INDEX index_spells_created_at ON spells(created_at);


CREATE TABLE nodes (
    node_id UUID NOT NULL PRIMARY KEY,
    content JSONB NOT NULL,
    parent_id UUID,
    parent_position INT,
    created_at TIMESTAMP NOT NULL DEFAULT utc_now(),
    created_by UUID,
    created_with UUID,
    CONSTRAINT fk_parent FOREIGN KEY(parent_id) REFERENCES nodes(node_id),
    CONSTRAINT fk_created_by FOREIGN KEY(created_by) REFERENCES users(user_id),
    CONSTRAINT fk_created_with FOREIGN KEY(created_with) REFERENCES spells(spell_id)
);

CREATE INDEX index_nodes_parent_id ON nodes(parent_id);
CREATE INDEX index_nodes_created_at ON nodes(created_at);
CREATE INDEX index_nodes_created_by ON nodes(created_by);

CREATE TABLE root_nodes (
    node_id UUID NOT NULL PRIMARY KEY,
    CONSTRAINT fk_node_id FOREIGN KEY(node_id) REFERENCES nodes(node_id)
);


CREATE TABLE recipes (
    recipe_id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    product UUID,
    ingredients UUID[] NOT NULL,
    CONSTRAINT fk_product FOREIGN KEY(product) REFERENCES spells(spell_id)
);

CREATE INDEX index_recipes_product ON recipes(product);


CREATE TABLE known_recipes (
    recipe_id UUID NOT NULL,
    user_id UUID NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT utc_now(),
    CONSTRAINT fk_recipe_id FOREIGN KEY(recipe_id) REFERENCES recipes(recipe_id),
    CONSTRAINT fk_user_id FOREIGN KEY(user_id) REFERENCES users(user_id)
);

CREATE INDEX index_known_recipes_recipe_id ON known_recipes(recipe_id);
CREATE INDEX index_known_recipes_user_id ON known_recipes(user_id);
CREATE INDEX index_known_recipes_created_at ON known_recipes(created_at);


CREATE TABLE inventory_contents (
    user_id UUID NOT NULL,
    spell_id UUID NOT NULL,
    amount INT NOT NULL,
    CONSTRAINT fk_user_id FOREIGN KEY(user_id) REFERENCES users(user_id),
    CONSTRAINT fk_spell_id FOREIGN KEY(spell_id) REFERENCES spells(spell_id)
);

CREATE INDEX index_inventory_contents_user_id ON inventory_contents(user_id);
CREATE INDEX index_inventory_contents_spell_id ON inventory_contents(spell_id);
