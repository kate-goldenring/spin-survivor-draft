#!/bin/bash

# Input file with player names (one name per line)
input_file=$1

# Season number
season=$2

# Output SQL file
output_file="players_migration_${season}.sql"

echo "Processing $input_file for season $season into $output_file"

# Clear the output file if it exists
true > "$output_file"

# Read each line from the input file
while IFS= read -r player_name; do
  # Escape single quotes in player names (for names like O'Connor)
  escaped_name=$(echo "$player_name" | sed "s/'/''/g")
  # Remove leading and trailing whitespace
  escaped_name=$(echo "$escaped_name" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')
  # Ensure name is not empty
    if [ -z "$escaped_name" ]; then
        echo "Skipping empty name"
        continue
    fi
  
  # Write the SQL INSERT statement to the output file
  echo "INSERT OR IGNORE INTO players (name, season, voted_out) VALUES ('$escaped_name', $season, NULL);" >> "$output_file"
done < "$input_file"

echo "SQL statements have been written to $output_file"
