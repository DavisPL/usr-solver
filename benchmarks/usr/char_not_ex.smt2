
(declare-const w String)
(declare-const x String)
(declare-const y String)

(assert (str.in_re x re.allchar))

(assert (str.in_re y re.allchar))

(assert (str.in_re w (re.++
    (re.inter
        (str.to_re x)
        (re.comp (str.to_re y))
    )
    (re.inter
        (str.to_re x)
        (str.to_re y)
    )
)))

(check-sat)
