; StringVar w
; Membership in intersection of
; w1(cat){0,3}w1
; w1(cat){2,5}w1
; w2w2w2w2w2

(declare-fun w1 () String)
(declare-fun w2 () String)
(declare-const x String)

(assert
    (str.in_re x
        (re.inter
            (re.++
                (str.to_re w1) ((_ re.loop 0 3) (str.to_re "cat")) (str.to_re w1)
            )
            (re.++
                (str.to_re w1) ((_ re.loop 0 3) (str.to_re "cat")) (str.to_re w1)
            )
            (re.++
                (str.to_re w2) (str.to_re w2) (str.to_re w2) (str.to_re w2) (str.to_re w2)
            )
        )
    )
)
(check-sat)