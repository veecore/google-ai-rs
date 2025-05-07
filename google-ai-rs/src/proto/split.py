import os
import re

def split_rust_file(input_path):
    """Recursively splits a Rust file with inline modules into separate files"""
    with open(input_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Find all module declarations using regex and manual brace counting
    modules = []
    pos = 0
    pattern = re.compile(r'pub\s+mod\s+(\w+)\s*\{', re.DOTALL)

    while True:
        match = pattern.search(content, pos)
        if not match:
            break

        mod_name = match.group(1)
        start = match.start()
        brace_pos = match.end() - 1  # Position of opening brace
        brace_depth = 1
        pos = brace_pos + 1

        # Find matching closing brace
        while pos < len(content) and brace_depth > 0:
            if content[pos] == '{':
                brace_depth += 1
            elif content[pos] == '}':
                brace_depth -= 1
            pos += 1

        end = pos  # Position after closing brace
        mod_content = content[brace_pos+1:pos-1].strip()
        modules.append((mod_name, mod_content, start, end))

    # Process modules in reverse order to maintain correct positions
    modules.sort(key=lambda x: x[2], reverse=True)
    modified_content = content
    
    for mod_name, mod_content, start, end in modules:
        # Replace inline module with declaration
        modified_content = modified_content[:start] + f'pub mod {mod_name};' + modified_content[end:]

        # Create module directory and write content
        mod_dir = os.path.join(os.path.dirname(input_path), mod_name)
        os.makedirs(mod_dir, exist_ok=True)
        mod_path = os.path.join(mod_dir, 'mod.rs')
        
        with open(mod_path, 'w', encoding='utf-8') as f:
            f.write(mod_content)

        # Recursively process new module file
        split_rust_file(mod_path)

    # Write modified original file
    with open(input_path, 'w', encoding='utf-8') as f:
        f.write(modified_content)

if __name__ == '__main__':
    # Replace with your actual generated file path
    split_rust_file('generativelanguage.rs')
    print("Successfully split Rust modules!")