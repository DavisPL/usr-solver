;---
; .NET regular expressions restricted to 7-bit characters
; membership in intersection of
; .*[a-z].*
; .*(\W|_).*
; (?(.*(012|123|234|345|456|567|678|789).*)[0-[0]]|.*)
; .{7,10}
;---
(declare-const x String)
(assert (str.in_re x
    (re.inter (re.inter (re.inter
        (re.++ (re.++ re.all (re.range "a" "z")) re.all)
        (re.++ (re.++ re.all (re.union (re.union (re.union (re.union (re.union (re.range (_ char #x0) "/") (re.range ":" "@")) (re.range "[" "^")) (str.to_re "`")) (re.range "{" (_ char #x7F))) (str.to_re "_"))) re.all))
        (re.union
            (re.inter (re.++ (re.++ re.all (re.union (re.union (re.union (re.union (re.union (re.union (re.union (str.to_re "012") (str.to_re "123")) (str.to_re "234")) (str.to_re "345")) (str.to_re "456")) (str.to_re "567")) (str.to_re "678")) (str.to_re "789"))) re.all) re.none)
            (re.inter (re.comp (re.++ (re.++ re.all (re.union (re.union (re.union (re.union (re.union (re.union (re.union (str.to_re "012") (str.to_re "123")) (str.to_re "234")) (str.to_re "345")) (str.to_re "456")) (str.to_re "567")) (str.to_re "678")) (str.to_re "789"))) re.all)) re.all)
        ))
        ((_ re.loop 7 10) re.allchar))
))
(check-sat)
