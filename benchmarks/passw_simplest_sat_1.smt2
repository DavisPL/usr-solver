;---
; .NET regular expressions restricted to 7-bit characters
; membership in intersection of
; (?(.*(012|123|234|345|456|567|678|789).*)[0-[0]]|.*)
; .{7,10}
;---
(declare-const x String)
(assert (str.in_re x
    (re.inter
        (re.comp (re.++ (re.++ re.all (re.union (re.union (re.union (re.union (re.union (re.union (re.union (str.to_re "012") (str.to_re "123")) (str.to_re "234")) (str.to_re "345")) (str.to_re "456")) (str.to_re "567")) (str.to_re "678")) (str.to_re "789"))) re.all))
        ((_ re.loop 7 10) re.allchar)
    )
))
(check-sat)
