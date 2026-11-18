-- SurrealQL language coverage fixture.
DEFINE TABLE person SCHEMAFULL;
DEFINE FIELD name ON person TYPE string;
DEFINE FIELD friends ON person TYPE array<record<person>>;

LET $name = 'Tobie';
CREATE person:tobie SET name = $name, active = true;
RELATE person:tobie->likes->person:jaime SET since = 2024;

SELECT id, name, friends FROM person
  WHERE name = $name AND active = true
  ORDER BY name DESC;
