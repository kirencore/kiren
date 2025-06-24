#!/bin/bash
cd /Users/mertcanaltin/Desktop/projects/kiren

# Remove debug and test files
rm -f debug_*.js
rm -f test_*.js  
rm -f quick_*.js
rm -f simple_*.js
rm -f build_and_test.sh
rm -f force_build.sh
rm -f run_test.py
rm -f static-import-test.txt

# Remove test directories
rm -rf test-kps/
rm -rf node_modules/

echo "Cleanup completed"