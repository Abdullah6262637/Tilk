işlev faktöriyel(n) {
    n <= 1 ise {
        döndür 1;
    }
    döndür n * faktöriyel(n - 1);
}

limit = 5;
i, 1'den limit'e dek artarak {
    yazdır(faktöriyel(i));
}
