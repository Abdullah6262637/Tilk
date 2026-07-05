#include "tilk_runtime.h"

TilkVal make_bos() {
    TilkVal v;
    v.type = VAL_BOS;
    return v;
}

TilkVal make_number(double n) {
    TilkVal v;
    v.type = VAL_NUMBER;
    v.val.number = n;
    return v;
}

TilkVal make_string(const char* s) {
    TilkVal v;
    v.type = VAL_STRING;
    v.val.string = strdup(s);
    return v;
}

TilkVal make_boolean(bool b) {
    TilkVal v;
    v.type = VAL_BOOLEAN;
    v.val.boolean = b;
    return v;
}

TilkVal make_hata(const char* msg) {
    TilkVal v;
    v.type = VAL_HATA;
    v.val.error = strdup(msg);
    return v;
}

void print_val_raw(TilkVal v) {
    switch (v.type) {
        case VAL_BOS: printf("boş"); break;
        case VAL_NUMBER: printf("%g", v.val.number); break;
        case VAL_STRING: printf("%s", v.val.string); break;
        case VAL_BOOLEAN: printf("%s", v.val.boolean ? "doğru" : "yanlış"); break;
        case VAL_ARRAY:
            printf("[");
            for (size_t i = 0; i < v.val.array.len; i++) {
                if (i > 0) printf(", ");
                print_val_raw(v.val.array.data[i]);
            }
            printf("]");
            break;
        case VAL_MAP:
            printf("{");
            for (size_t i = 0; i < v.val.map.len; i++) {
                if (i > 0) printf(", ");
                printf("\"%s\": ", v.val.map.keys[i]);
                print_val_raw(v.val.map.values[i]);
            }
            printf("}");
            break;
        case VAL_HATA: printf("Hata: %s", v.val.error); break;
    }
}

TilkVal yazdir(TilkVal v) {
    print_val_raw(v);
    printf("\n");
    return make_bos();
}
TilkVal yazd_r(TilkVal v) { return yazdir(v); }

TilkVal boyut(TilkVal v) {
    if (v.type == VAL_ARRAY) return make_number(v.val.array.len);
    if (v.type == VAL_MAP) return make_number(v.val.map.len);
    if (v.type == VAL_STRING) return make_number(strlen(v.val.string));
    return make_number(0);
}

TilkVal ekle(TilkVal arr, TilkVal item) {
    if (arr.type == VAL_ARRAY) {
        TilkArray* a = &arr.val.array;
        if (a->len >= a->capacity) {
            a->capacity = a->capacity == 0 ? 4 : a->capacity * 2;
            a->data = realloc(a->data, a->capacity * sizeof(TilkVal));
        }
        a->data[a->len++] = item;
    }
    return make_bos();
}

TilkVal hata_firlat(TilkVal v) {
    if (v.type == VAL_STRING) return make_hata(v.val.string);
    return make_hata("Hata");
}
TilkVal hata_f_rlat(TilkVal v) { return hata_firlat(v); }

TilkVal kok(TilkVal v) {
    if (v.type == VAL_NUMBER) {
        if (v.val.number < 0) return make_hata("Negatif sayının karekökü alınamaz");
        return make_number(sqrt(v.val.number));
    }
    return make_hata("Sayısal değer bekleniyordu");
}
TilkVal karekok(TilkVal v) { return kok(v); }

TilkVal us(TilkVal base, TilkVal exp) {
    if (base.type == VAL_NUMBER && exp.type == VAL_NUMBER) {
        return make_number(pow(base.val.number, exp.val.number));
    }
    return make_hata("Sayısal değer bekleniyordu");
}
TilkVal ust(TilkVal base, TilkVal exp) { return us(base, exp); }

TilkVal mutlak(TilkVal v) {
    if (v.type == VAL_NUMBER) return make_number(fabs(v.val.number));
    return make_hata("Sayısal değer bekleniyordu");
}

TilkVal simdi() {
    return make_number((double)time(NULL));
}
TilkVal simdi_zaman() { return simdi(); }

TilkVal uyku(TilkVal ms) {
    if (ms.type == VAL_NUMBER) {
        #ifdef _WIN32
        Sleep((DWORD)ms.val.number);
        #else
        usleep((useconds_t)(ms.val.number * 1000));
        #endif
    }
    return make_bos();
}

TilkVal kanal() {
    TilkVal v;
    v.type = VAL_CHANNEL;
    v.val.channel.data = malloc(4 * sizeof(TilkVal));
    v.val.channel.front = 0;
    v.val.channel.back = 0;
    v.val.channel.capacity = 4;
    return v;
}

TilkVal dosya_oku(TilkVal path) {
    if (path.type != VAL_STRING) return make_hata("Dosya yolu metin olmalıdır");
    FILE* f = fopen(path.val.string, "r");
    if (!f) return make_hata("Dosya okunamadı");
    fseek(f, 0, SEEK_END);
    long len = ftell(f);
    fseek(f, 0, SEEK_SET);
    char* buf = malloc(len + 1);
    fread(buf, 1, len, f);
    buf[len] = '\0';
    fclose(f);
    TilkVal v = make_string(buf);
    free(buf);
    return v;
}

TilkVal dosya_yaz(TilkVal path, TilkVal content) {
    if (path.type != VAL_STRING || content.type != VAL_STRING) return make_hata("Dosya yolu ve içerik metin olmalıdır");
    FILE* f = fopen(path.val.string, "w");
    if (!f) return make_hata("Dosya yazılamadı");
    fputs(content.val.string, f);
    fclose(f);
    return make_boolean(true);
}

TilkVal dosya_sil(TilkVal path) {
    if (path.type != VAL_STRING) return make_hata("Dosya yolu metin olmalıdır");
    if (remove(path.val.string) == 0) return make_boolean(true);
    return make_hata("Dosya silinemedi");
}

TilkVal add_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_number(a.val.number + b.val.number);
    if (a.type == VAL_STRING && b.type == VAL_STRING) {
        char* buf = malloc(strlen(a.val.string) + strlen(b.val.string) + 1);
        strcpy(buf, a.val.string);
        strcat(buf, b.val.string);
        TilkVal v = make_string(buf);
        free(buf);
        return v;
    }
    return make_hata("Uyumsuz tipler");
}
TilkVal sub_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_number(a.val.number - b.val.number);
    return make_hata("Uyumsuz tipler");
}
TilkVal mul_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_number(a.val.number * b.val.number);
    return make_hata("Uyumsuz tipler");
}
TilkVal div_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) {
        if (b.val.number == 0) return make_hata("Sıfıra bölme hatası");
        return make_number(a.val.number / b.val.number);
    }
    return make_hata("Uyumsuz tipler");
}
TilkVal mod_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_number(fmod(a.val.number, b.val.number));
    return make_hata("Uyumsuz tipler");
}
TilkVal eq_val(TilkVal a, TilkVal b) {
    if (a.type != b.type) return make_boolean(false);
    switch (a.type) {
        case VAL_BOS: return make_boolean(true);
        case VAL_NUMBER: return make_boolean(a.val.number == b.val.number);
        case VAL_STRING: return make_boolean(strcmp(a.val.string, b.val.string) == 0);
        case VAL_BOOLEAN: return make_boolean(a.val.boolean == b.val.boolean);
        default: return make_boolean(false);
    }
}
TilkVal ne_val(TilkVal a, TilkVal b) { return make_boolean(!eq_val(a, b).val.boolean); }
TilkVal lt_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_boolean(a.val.number < b.val.number);
    return make_boolean(false);
}
TilkVal gt_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_boolean(a.val.number > b.val.number);
    return make_boolean(false);
}
TilkVal le_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_boolean(a.val.number <= b.val.number);
    return make_boolean(false);
}
TilkVal ge_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_NUMBER && b.type == VAL_NUMBER) return make_boolean(a.val.number >= b.val.number);
    return make_boolean(false);
}
TilkVal and_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_BOOLEAN && b.type == VAL_BOOLEAN) return make_boolean(a.val.boolean && b.val.boolean);
    return make_boolean(false);
}
TilkVal or_val(TilkVal a, TilkVal b) {
    if (a.type == VAL_BOOLEAN && b.type == VAL_BOOLEAN) return make_boolean(a.val.boolean || b.val.boolean);
    return make_boolean(false);
}

TilkVal index_val(TilkVal target, TilkVal idx) {
    if (target.type == VAL_ARRAY && idx.type == VAL_NUMBER) {
        size_t i = (size_t)idx.val.number;
        if (i < target.val.array.len) return target.val.array.data[i];
        return make_hata("Dizi sınırları dışında erişim");
    }
    if (target.type == VAL_MAP && idx.type == VAL_STRING) {
        TilkMap* m = &target.val.map;
        for (size_t i = 0; i < m->len; i++) {
            if (strcmp(m->keys[i], idx.val.string) == 0) return m->values[i];
        }
        return make_bos();
    }
    if (target.type == VAL_CHANNEL) {
        TilkChannel* c = &target.val.channel;
        if (c->front == c->back) return make_bos();
        TilkVal val = c->data[c->front];
        c->front++;
        return val;
    }
    return make_hata("İndeksleme hatası");
}

TilkVal index_assign(TilkVal target, TilkVal idx, TilkVal val) {
    if (target.type == VAL_ARRAY && idx.type == VAL_NUMBER) {
        size_t i = (size_t)idx.val.number;
        if (i < target.val.array.len) {
            target.val.array.data[i] = val;
            return make_bos();
        }
        return make_hata("Dizi sınırları dışında yazma");
    }
    if (target.type == VAL_MAP && idx.type == VAL_STRING) {
        TilkMap* m = &target.val.map;
        for (size_t i = 0; i < m->len; i++) {
            if (strcmp(m->keys[i], idx.val.string) == 0) {
                m->values[i] = val;
                return make_bos();
            }
        }
        if (m->len >= m->capacity) {
            m->capacity = m->capacity == 0 ? 4 : m->capacity * 2;
            m->keys = realloc(m->keys, m->capacity * sizeof(char*));
            m->values = realloc(m->values, m->capacity * sizeof(TilkVal));
        }
        m->keys[m->len] = strdup(idx.val.string);
        m->values[m->len++] = val;
        return make_bos();
    }
    if (target.type == VAL_CHANNEL) {
        TilkChannel* c = &target.val.channel;
        if (c->back >= c->capacity) {
            c->capacity *= 2;
            c->data = realloc(c->data, c->capacity * sizeof(TilkVal));
        }
        c->data[c->back++] = val;
        return make_bos();
    }
    return make_hata("İndeksleme hatası");
}

TilkVal create_array(size_t len, ...) {
    TilkVal v;
    v.type = VAL_ARRAY;
    v.val.array.data = len == 0 ? NULL : malloc(len * sizeof(TilkVal));
    v.val.array.len = len;
    v.val.array.capacity = len;
    va_list args;
    va_start(args, len);
    for (size_t i = 0; i < len; i++) {
        v.val.array.data[i] = va_arg(args, TilkVal);
    }
    va_end(args);
    return v;
}

TilkVal create_map(size_t pair_count, ...) {
    TilkVal v;
    v.type = VAL_MAP;
    v.val.map.keys = pair_count == 0 ? NULL : malloc(pair_count * sizeof(char*));
    v.val.map.values = pair_count == 0 ? NULL : malloc(pair_count * sizeof(TilkVal));
    v.val.map.len = pair_count;
    v.val.map.capacity = pair_count;
    va_list args;
    va_start(args, pair_count);
    for (size_t i = 0; i < pair_count; i++) {
        TilkVal k = va_arg(args, TilkVal);
        TilkVal val = va_arg(args, TilkVal);
        v.val.map.keys[i] = strdup(k.val.string);
        v.val.map.values[i] = val;
    }
    va_end(args);
    return v;
}

TilkVal neg_val(TilkVal v) {
    if (v.type == VAL_NUMBER) return make_number(-v.val.number);
    return make_hata("Negatif islem sadece sayilarla yapilabilir");
}

TilkVal not_val(TilkVal v) {
    if (v.type == VAL_BOOLEAN) return make_boolean(!v.val.boolean);
    return make_hata("Mantiksal degil islemi sadece mantiksal degerlerle yapilabilir");
}
