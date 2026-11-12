import random
import argparse
from contextlib import contextmanager

class IndentedBuilder:
    def __init__(self, indent_char="    "):
        self.lines = []
        self.level = 0
        self.indent_char = indent_char

    def write(self, text=""):
        """Appends a line with the correct current indentation."""
        if text:
            indentation = self.indent_char * self.level
            self.lines.append(f"{indentation}{text}")
        else:
            self.lines.append("")

    @contextmanager
    def block(self, header=""):
        """Context manager that outputs a header, increases indentation, 
        and automatically appends a closing parenthesis ')' when leaving."""
        if header:
            self.write(header)
        self.level += 1
        try:
            yield self
        finally:
            self.level -= 1
            self.write(")")

    def build(self):
        """Returns the final multi-line string."""
        return "\n".join(self.lines)

def single_substr_expr(index: int, length: int, target: str) -> str:
    """Generates the regex expression block for a single substring constraint."""
    b = IndentedBuilder(indent_char="    ")
    
    i = index
    L = length
    sum_il = i + L

    def ac(n):
        """Returns a list of re.allchar tokens, or empty list if n <= 0."""
        return ["re.allchar"] * n if n > 0 else []

    def re_concat(*items):
        """Combines items and wraps in re.++ only if necessary."""
        flat = []
        for item in items:
            if isinstance(item, list):
                flat.extend(item)
            elif item:
                flat.append(item)
        
        if not flat:
            return ""
        if len(flat) == 1:
            return flat[0]
        return f"(re.++ {' '.join(flat)})"

    with b.block("(re.union"):
        # When |s1|>=i+j
        with b.block("(re.++"):
            with b.block("(re.inter"):
                b.write("(str.to_re s1)")
                b.write(re_concat(ac(i), f'(str.to_re "{target}")', "(re.* re.allchar)"))
            
            with b.block("(re.inter"):
                b.write(f'(str.to_re "{target}")')
                if L>0:
                    b.write(re_concat(ac(L)))
                else:
                    b.write('(str.to_re "")')
            
            with b.block("(re.inter"):
                b.write("(str.to_re s1)")
                b.write(re_concat(ac(sum_il), "(re.* re.allchar)"))

        # When |s1|>=1 but <j
        with b.block("(re.++"):
            with b.block("(re.inter"):
                b.write("(str.to_re s1)")
                b.write(re_concat(ac(i), f'(str.to_re "{target}")'))
            
            with b.block("(re.inter"):
                b.write("(str.to_re s1)")
                comp_content = re_concat(ac(L), "(re.* re.allchar)")
                comp_expr = f"(re.comp {comp_content})"
                b.write(re_concat(ac(i), comp_expr))

        # When |s1|<i
        with b.block("(re.++"):
            with b.block("(re.inter"):
                b.write("(str.to_re s1)")
                comp_content_2 = re_concat(ac(i), "(re.* re.allchar)")
                comp_expr_2 = f"(re.comp {comp_content_2})"
                b.write(comp_expr_2)
            
            with b.block("(re.inter"):
                b.write(f'(str.to_re "{target}")')
                b.write('(str.to_re "")')

    return b.build()

def generate_multi_constraints(constraints) -> str:
    b = IndentedBuilder(indent_char="    ")
    
    b.write("(declare-const s1 String)")
    b.write("(declare-const y String)")
    b.write()
    
    for i, L, target in constraints:
        b.write(f';(assert (= "{target}" (str.substr s1 {i} {L})))')
    b.write()

    expressions = [single_substr_expr(i, L, t) for i, L, t in constraints]

    with b.block("(assert"):
        with b.block("(str.in_re y"):
            if len(expressions) == 1:
                for line in expressions[0].split("\n"):
                    b.write(line)
            else:
                with b.block("(re.++"):
                    for expr in expressions:
                        for line in expr.split("\n"):
                            b.write(line)
    b.write("(check-sat)")

    return b.build()

def generate_random_substr_constraints() -> str:
    pool="abcdefghijklmnopqrstuvwxyz0123456789"
    num_constraints=random.randint(1,6)
    constraints=[]
    for i in range(num_constraints):
        index=random.randint(0,20)
        length=random.randint(0,0)
        rand_string="".join(random.choices(pool,k=length))
        if random.random() > 0.8:
            length = random.randint(0,0)
        constraints.append((index,length,rand_string))
    return generate_multi_constraints(constraints)


if __name__ == "__main__":
    print(generate_multi_constraints([(2,0,""),(0,2,"aa"),(1,1,"b")]))