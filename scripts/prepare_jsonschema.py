import os

def process_json_schema_files(input_folder: str, output_folder: str):
    """
    Reads files ending with '.jsonschema.json' from input_folder
    and writes them to output_folder with the 'jsonschema' suffix removed.
    Also replaces all instances of ".jsonschema.json" in file content with ".json".
    """
    # Create output folder if it doesn't exist
    if not os.path.exists(output_folder):
        os.makedirs(output_folder)

    # Get all files in the input folder
    for filename in os.listdir(input_folder):
        if filename.endswith('.schema.json'):
            input_path = os.path.join(input_folder, filename)
            # Remove the 'jsonschema' suffix from the filename
            new_filename = filename.replace('.schema.json', '.json')
            output_path = os.path.join(output_folder, new_filename)

            # Read the content, replace instances of ".jsonschema.json" with ".json"
            with open(input_path, 'r') as file:
                content = file.read()
                updated_content = content.replace('schema.json', 'json')

            # Write the updated content to the new file
            with open(output_path, 'w') as file:
                file.write(updated_content)

            print(f"Processed: {filename} -> {new_filename}")

if __name__ == "__main__":
    # Define input and output folders
    input_folder = "app/src/gen/jsonschema"
    output_folder = "tmp_schemas"

    process_json_schema_files(input_folder, output_folder)
