from shutil import copyfile
import sys
import subprocess
import os

def copy_dependencies(target_executable, target_folder):
    print("[*] Gathering dependencies of " + target_executable)
    cmd = "lddtree -l " + target_executable
    proc = subprocess.Popen(cmd.split(" "), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if proc.wait() != 0:
        raise Exception(proc.stderr.read())

    dependencies = proc.stdout.read().decode("utf-8").rstrip().split("\n")

    skip_libraries = ['ld-linux-x86-64.so.2', 'http_fuzz_api.so', 'libhdhunter_rt.so']

    i = 1
    for d in dependencies[1:]:
        if os.path.basename(d) not in skip_libraries:
            print("[+] Copying " + d + " to " + target_folder)
            copyfile(d, "%s/%s"%(target_folder, os.path.basename(d)))

if __name__ == '__main__':
    copy_dependencies(sys.argv[1], sys.argv[2])
