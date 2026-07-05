#ifndef TILK_RUNTIME_H
#define TILK_RUNTIME_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <math.h>
#include <time.h>
#include <stdarg.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <unistd.h>
#endif

typedef enum {
    VAL_BOS,
    VAL_NUMBER,
    VAL_STRING,
    VAL_BOOLEAN,
    VAL_ARRAY,
    VAL_MAP,
    VAL_CHANNEL,
    VAL_HATA
} TilkType;

struct TilkVal;

typedef struct {
    struct TilkVal* data;
    size_t front;
    size_t back;
    size_t capacity;
} TilkChannel;

typedef struct {
    struct TilkVal* data;
    size_t len;
    size_t capacity;
} TilkArray;

typedef struct {
    char** keys;
    struct TilkVal* values;
    size_t len;
    size_t capacity;
} TilkMap;

typedef struct TilkVal {
    TilkType type;
    union {
        double number;
        char* string;
        bool boolean;
        TilkArray array;
        TilkMap map;
        TilkChannel channel;
        char* error;
    } val;
} TilkVal;

TilkVal make_bos();
TilkVal make_number(double n);
TilkVal make_string(const char* s);
TilkVal make_boolean(bool b);
TilkVal make_hata(const char* msg);

void print_val_raw(TilkVal v);
TilkVal yazdir(TilkVal v);
TilkVal yazd_r(TilkVal v);
TilkVal boyut(TilkVal v);
TilkVal ekle(TilkVal arr, TilkVal item);
TilkVal hata_firlat(TilkVal v);
TilkVal hata_f_rlat(TilkVal v);
TilkVal kok(TilkVal v);
TilkVal karekok(TilkVal v);
TilkVal us(TilkVal base, TilkVal exp);
TilkVal ust(TilkVal base, TilkVal exp);
TilkVal mutlak(TilkVal v);
TilkVal simdi();
TilkVal simdi_zaman();
TilkVal uyku(TilkVal ms);
TilkVal kanal();
TilkVal dosya_oku(TilkVal path);
TilkVal dosya_yaz(TilkVal path, TilkVal content);
TilkVal dosya_sil(TilkVal path);

TilkVal add_val(TilkVal a, TilkVal b);
TilkVal sub_val(TilkVal a, TilkVal b);
TilkVal mul_val(TilkVal a, TilkVal b);
TilkVal div_val(TilkVal a, TilkVal b);
TilkVal mod_val(TilkVal a, TilkVal b);
TilkVal eq_val(TilkVal a, TilkVal b);
TilkVal ne_val(TilkVal a, TilkVal b);
TilkVal lt_val(TilkVal a, TilkVal b);
TilkVal gt_val(TilkVal a, TilkVal b);
TilkVal le_val(TilkVal a, TilkVal b);
TilkVal ge_val(TilkVal a, TilkVal b);
TilkVal and_val(TilkVal a, TilkVal b);
TilkVal or_val(TilkVal a, TilkVal b);
TilkVal index_val(TilkVal target, TilkVal idx);
TilkVal index_assign(TilkVal target, TilkVal idx, TilkVal val);
TilkVal create_array(size_t len, ...);
TilkVal create_map(size_t pair_count, ...);
TilkVal neg_val(TilkVal v);
TilkVal not_val(TilkVal v);

#endif
