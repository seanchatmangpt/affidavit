import os

header = """// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
"""

directories = ['src', 'tests', 'examples', 'benches']
updated_count = 0

for root_dir in directories:
    if not os.path.exists(root_dir):
        continue
    for root, dirs, files in os.walk(root_dir):
        for file in files:
            if file.endswith('.rs'):
                path = os.path.join(root, file)
                with open(path, 'r', encoding='utf-8') as f:
                    try:
                        content = f.read()
                    except UnicodeDecodeError:
                        continue
                
                if 'SPDX-License-Identifier' not in content:
                    with open(path, 'w', encoding='utf-8') as f:
                        f.write(header + content)
                    updated_count += 1

print(f"Updated {updated_count} files.")
