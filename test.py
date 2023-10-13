import time
import argparse


parser = argparse.ArgumentParser(description='Test')
parser.add_argument('-a', type=int, default=0, help='A')
parser.add_argument('-b', type=str, default='b', help='B')
args = parser.parse_args()

if args.a == 1:
    print('A')
if args.b == 'b':
    print('B')

print(1)
time.sleep(5)
print(2)
time.sleep(5)
print(3)
