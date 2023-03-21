add16 r1, r2, r3, r4
add16 r5, r6, r7, r8

@add16 %l1, %h1, %l2, %h2
    add %l1, %l2
    adc %h1, %h2
end

@add2 %r1, %r2
    add %r1, %r2
end