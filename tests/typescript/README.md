# TypeScript Test Suite for Kiren

Bu klasör Kiren'in SWC entegrasyonu ile TypeScript desteğini test eden dosyaları içerir.

## Test Dosyaları

### 1. `basit.ts`
- Temel TypeScript özellikleri
- Interface ve function tanımları
- Generic fonksiyonlar
- Type annotations

### 2. `simple_test.ts` 
- Kullanıcı dostu basit test
- Interface, optional properties
- Union types, array methods
- Generic utility functions

### 3. `basic_features.ts`
- Temel TypeScript features
- Class ve interface kullanımı
- Type aliases
- Method chaining

### 4. `generics_test.ts`
- Generic class ve interface testleri
- Constrained generics
- Multiple type parameters
- Keyof operator kullanımı

### 5. `comprehensive_test.ts`
- Kapsamlı TypeScript test suite
- Repository pattern
- Async/Promise handling
- Complex type operations
- Inheritance ve composition

## Testleri Çalıştırma

```bash
# Temel test
./target/debug/kiren tests/typescript/basit.ts

# Basit test
./target/debug/kiren tests/typescript/simple_test.ts

# Generics test
./target/debug/kiren tests/typescript/generics_test.ts

# Kapsamlı test
./target/debug/kiren tests/typescript/comprehensive_test.ts
```

## Desteklenen TypeScript Özellikleri

✅ **Interfaces** - Type definitions ve contracts
✅ **Classes** - OOP support
✅ **Generics** - Type parameterization  
✅ **Union Types** - Multiple type options
✅ **Optional Properties** - `property?: type`
✅ **Type Aliases** - Custom type definitions
✅ **Async/Await** - Promise-based operations
✅ **Array Methods** - Typed array operations
✅ **Type Annotations** - Explicit typing
✅ **Inheritance** - Class extension
✅ **Method Overloading** - Multiple signatures

## SWC Strip-Only Mode Kısıtlamaları

❌ **Enums** - Object literal kullanın
❌ **Parameter Properties** - Constructor'da explicit assignment
❌ **Decorators** - Modern JavaScript patterns kullanın
❌ **Namespace** - ES6 modules kullanın

## Performans

SWC ile TypeScript transpilation çok hızlı (regex-based fallback ile):
- Ortalama transpilation süresi: ~1-2ms
- Büyük dosyalar için bile optimize edilmiş
- Production-ready performance

## Hata Ayıklama

Eğer bir TypeScript dosyası çalışmıyorsa:

1. SWC strip-only mode kısıtlamalarını kontrol edin
2. Global scope name conflicts olup olmadığını kontrol edin
3. Dosya encoding'inin UTF-8 olduğundan emin olun
4. Fallback regex transpiler otomatik devreye girer