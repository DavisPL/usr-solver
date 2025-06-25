;---
; .NET regular expressions restricted to 7-bit characters
; membership in intersection of
; (?(.*(012|123|234|345|456|567|678|789).*)[0-[0]]|.*)
; .{7,10}
;---
(declare-const x String)
(assert (str.in_re x
    (re.inter
        (re.comp (re.++ (re.++ re.all   (re.union (str.to_re "012") (str.to_re "123")) ) re.all))
        ((_ re.loop 5 7) re.allchar)
    )
))
(check-sat)
