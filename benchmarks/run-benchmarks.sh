#!/bin/bash

echo "🚀 Kiren vs Node.js Benchmark Testi"
echo "===================================="

# Node.js versiyonunu kontrol et
if ! command -v node &> /dev/null; then
    echo "❌ Node.js bulunamadı. Lütfen Node.js'i yükleyin."
    exit 1
fi

echo "Node.js version: $(node --version)"
echo "Kiren version: 0.1.0"
echo ""

# Startup Time Benchmark
echo "📊 1. Startup Time Benchmark"
echo "----------------------------"

echo "Testing Kiren startup..."
kiren_startup_time=$(
    { time ../target/release/kiren startup-test.js; } 2>&1 | grep real | awk '{print $2}'
)

echo "Testing Node.js startup..."
node_startup_time=$(
    { time node startup-test.js; } 2>&1 | grep real | awk '{print $2}'
)

echo "Kiren startup: $kiren_startup_time"
echo "Node.js startup: $node_startup_time"
echo ""

# Fibonacci Benchmark  
echo "📊 2. Fibonacci Calculation (CPU Intensive)"
echo "-------------------------------------------"

echo "Testing Kiren fibonacci..."
../target/release/kiren fibonacci-test.js

echo ""
echo "Testing Node.js fibonacci..."
node fibonacci-test.js

echo ""

# Loop Benchmark
echo "📊 3. Loop Performance"
echo "---------------------"

echo "Testing Kiren loop..."
../target/release/kiren loop-test.js

echo ""
echo "Testing Node.js loop..."
node loop-test.js

echo ""
echo "✅ Benchmark tamamlandı!"