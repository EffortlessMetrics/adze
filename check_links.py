import re
import os
import subprocess

def check_links():
    grep_process = subprocess.Popen(
        ['grep', '-rE', r'\[.*\]\((?!https?:\/\/).*\)', 'docs/'],
        stdout=subprocess.PIPE
    )
    stdout, _ = grep_process.communicate()

    if grep_process.returncode not in [0, 1]:
        print("Grep failed")
        return

    for line in stdout.decode('utf-8').splitlines():
        try:
            filepath, match = line.split(':', 1)
            link_match = re.search(r'\[.*\]\((.*)\)', match)
            if link_match:
                link = link_match.group(1)
                if '#' in link:
                    link = link.split('#')[0]

                if not link: # Skip anchor links
                    continue

                link_path = os.path.join(os.path.dirname(filepath), link)
                if not os.path.exists(link_path):
                    print(f"Broken link in {filepath}: {link} -> {link_path}")
        except ValueError:
            # Handle lines that don't have a link after the first colon
            pass

if __name__ == "__main__":
    check_links()
