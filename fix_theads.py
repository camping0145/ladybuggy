import os

def fix_thread_lengths():
    target_files = ['render.c', 'main.rs'] # checking core source layouts
    for file in os.listdir('.'):
        if file.endswith('.c') or file.endswith('.h'):
            try:
                with open(file, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()
                
                # Shorten the long thread names to fit the strict 16-byte Linux limit
                updated = content.replace("ladybuggy:render:", "lb:render:")
                updated = updated.replace("ladybuggy:pgo:", "lb:pgo:")
                
                if updated != content:
                    with open(file, 'w', encoding='utf-8') as f:
                        f.write(updated)
                    print(f"Fixed thread name lengths in: {file}")
            except Exception as e:
                continue

if __name__ == '__main__':
    fix_thread_lengths()
