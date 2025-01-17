import sys
import os
import argparse

argparser = argparse.ArgumentParser(description='Generate Java filters for Witcher-java')
argparser.add_argument('classpath', help='The root directory of the classpath')
argparser.add_argument('target_file', help='The target file to write the filters to')
argparser.add_argument('-a', '--append', action='store_true', help='Append to the target file instead of overwriting it')
argparser.add_argument('-f', '--filter', nargs='?', help='The filter to apply to the classes')

blacklist = ['module-info.class']

def generate_javafilters(classpath, target_file, filter=None, append=False):
    with open(target_file, 'w' if not append else 'a+') as out:
        for root, subdirs, files in os.walk(classpath):
            for f in files:
                if f in blacklist:
                    continue
                path = root[len(classpath):].replace('/', '.').strip('.')
                if (filter is None or filter in f or filter in path) and f.endswith('.class'):
                    out.write('+' + path + '.' + f.replace('.class', '') + '\n')

if __name__ == '__main__':
    args = argparser.parse_args()
    print(args.filter)
    if args.append:
        generate_javafilters(args.classpath, args.target_file, args.filter, True)
    else:
        generate_javafilters(args.classpath, args.target_file, args.filter)