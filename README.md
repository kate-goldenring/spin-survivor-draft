# Survivor Draft Spin Application

## Example from Season 46

Run application, setting a username and password for the SQLite explorer, the survivor season, and the drafting deadline in UTC.

```sh
SPIN_VARIABLE_DRAFT_DEADLINE="2025-09-03T03:00:00" SPIN_VARIABLE_SQLITE_USERNAME=kate SPIN_VARIABLE_SQLITE_PASSWORD=pw SPIN_VARIABLE_SEASON=47 spin build -u --sqlite="@migration.up.sql"
```

Initialize players

1. Get a list of all players and add to a text file with one player per line
2. Generate the migration file for the given list of players and season. Run this migration on your database directly using `sqlite3` or during the next `spin up`.

    ```sh
    ./generate-players-migration.sh players-46.txt 46
    sqlite3 .spin/sqlite_db.db < players_migration_46.sql
    ```


Join draft

1. Use UI form at http://127.0.0.1:3000
2. CLI

    ```sh
    curl -X POST -d '{ "drafter": "Foo Bar", "players": ["Some One", "Some Two", "Some Three"]}' http://127.0.0.1:3000/api/join/
    ```

    Change draft (so long as deadline has not passed)

    ```sh
    curl -X POST -d '{ "drafter": "Foo Bar", "players": ["Another One", "Another Two", "Another Three"]}' http://127.0.0.1:3000/api/join/
    ```

Vote out a player

```sh
curl -X POST -d "Jelinsky" http://127.0.0.1:3000/api/vote-out/
```