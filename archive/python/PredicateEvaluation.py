from classes import Literal, EmptySet, CharVar, StringVar, AndPred, UnionPred, Equals, Not, UnionFind, EqualLength, StringIndex
import itertools

def flatten_and_predicates(predicate):
    """Recursively flatten nested AndPred instances"""
    if not isinstance(predicate, AndPred):
        return [predicate]
        
    flattened = []
    for p in predicate.predicates:
        if isinstance(p, AndPred):
            flattened.extend(flatten_and_predicates(p))
        else:
            flattened.append(p)
    return flattened

def evaluate(predicate, unionFind = None):
    if unionFind is None:
        unionFind = UnionFind()
    alphabet = {"a", "b"}
    if isinstance(predicate, AndPred):
        all_predicates = flatten_and_predicates(predicate)
        #print(all_predicates)

        FinalPreds = []
        NotEqualityPreds = set()
        LengthPreds = dict()
        NotAllowedLengths = dict()
        Equalities = set()
        for p in all_predicates:
            if isinstance(p, Not):
                NotEqualityPreds.add(p)
            elif isinstance(p, Equals):
                Equalities.add(p)
            elif isinstance(p, EqualLength):
                temp = LengthPreds.get(p.left.name, None)
                if temp is not None and temp != p.right:
                    return EmptySet()
                LengthPreds[p.left.name] = p.right
            elif isinstance(p, EmptySet):
                return EmptySet()

            #elif isinstance(p, Literal) and p.value == "":
        for p in Equalities:
            #print(p.left, p.right)
            try:
                if isinstance(unionFind.union(p.left, p.right), EmptySet):
                    return EmptySet()
                else:
                    FinalPreds.append(p)
            except:
                return EmptySet()

        cantEqualChars = dict()
        for p in NotEqualityPreds:
            q = p.predicate
            if isinstance(q, Equals):
                ufLeft = unionFind.find(q.left)
                ufRight = unionFind.find(q.right)
                ufLeftO = unionFind.stringToObject(unionFind.find(q.left))
                ufRightO = unionFind.stringToObject(unionFind.find(q.right))
                #ufRight = unionFind.find(q.right)
                if ufLeft == ufRight:
                    return EmptySet()
                elif not(isinstance(ufLeftO, Literal)) or not(isinstance(ufRightO, Literal)):
                    if isinstance(ufRightO, Literal):
                        cantEqualChars[ufLeft] = cantEqualChars.get(ufLeft, set())
                        cantEqualChars[ufLeft].add(ufRightO.value)
                    FinalPreds.append(p)
            else:
                flag = False
                for key, value in unionFind.parent.items():
                    tempKey = unionFind.stringToObject(key)
                    tempValue = unionFind.stringToObject(value)
                    if isinstance(tempKey, StringIndex) and tempKey.stringVar == q.left.name and tempKey.index >= q.right:
                        if not(isinstance(tempValue, StringIndex) and tempValue.stringVar == tempKey.stringVar and tempValue.index == tempKey.index):
                            flag = True
                            break
                if flag:
                    continue
                #if not q.right:
                FinalPreds.append(p)
        for key, val in cantEqualChars.items():
            if all(char in val for char in alphabet):
                return EmptySet()
        for p, q in LengthPreds.items():
            if q in NotAllowedLengths.get(p, set()):
                return EmptySet()
            #pObj = unionFind.stringToObject(unionFind.find(StringIndex(p.left, p.right)))
            #print(unionFind.parent, p.left.name, p.right)
            for key, value in unionFind.parent.items():
                tempKey = unionFind.stringToObject(key)
                tempValue = unionFind.stringToObject(value)
                if isinstance(tempKey, StringIndex) and tempKey.stringVar == p and tempKey.index >= q:
                    if not(isinstance(tempValue, StringIndex) and tempValue.stringVar == tempKey.stringVar and tempValue.index == tempKey.index):
                        return EmptySet()
            #if not(isinstance(pObj, StringIndex) and pObj.stringVar == p.left.name and p.right == pObj.index):
                #return EmptySet()
            FinalPreds.append(EqualLength(StringVar(p), q))
        if len(FinalPreds) == 1:
            return FinalPreds[0]
        elif len(FinalPreds) == 0:
            return Literal("")
        return AndPred(*FinalPreds)
    elif isinstance(predicate, UnionPred):
        finalSet = []
        for p in predicate.predicates:
            #print(p)
            pE = evaluate(p, UnionFind())
            if isinstance(pE, Literal) and pE.value == "":
                return Literal("")
            elif isinstance(pE, EmptySet):
                continue
            else:
                finalSet.append(pE)
        if len(finalSet) == 1:
            return finalSet[0]
        elif len(finalSet) == 0:
            return EmptySet()
        return UnionPred(*finalSet)
    return predicate


def convertToDNF(predicate):
    if isinstance(predicate, Equals):
        return predicate
    elif isinstance(predicate, UnionPred):
        # Convert each child to DNF first
        dnf_children = [convertToDNF(child) for child in predicate.predicates]
        # Flatten nested UnionPreds
        flattened = []
        for child in dnf_children:
            if isinstance(child, UnionPred):
                flattened.extend(child.predicates)
            else:
                flattened.append(child)
        return UnionPred(*flattened)
    elif isinstance(predicate, AndPred):
        # Convert each child to DNF first
        dnf_children = [convertToDNF(child) for child in predicate.predicates]
        # Flatten nested ANDs before distribution
        flattened = []
        for child in dnf_children:
            if isinstance(child, AndPred):
                flattened.extend(child.predicates)
            else:
                flattened.append(child)
        # Now distribute
        return distributeOrs(flattened)
    elif isinstance(predicate, Not):
        if isinstance(predicate.predicate, AndPred):
            return UnionPred(*[convertToDNF(Not(child)) for child in predicate.predicate.predicates])
        elif isinstance(predicate.predicate, UnionPred):
            return AndPred(*[convertToDNF(Not(child)) for child in predicate.predicate.predicates])
        elif isinstance(predicate.predicate, Not):
            return convertToDNF(predicate.predicate.predicate)
        elif isinstance(predicate.predicate, Literal) and predicate.predicate.value == "":
            return EmptySet()
        elif isinstance(predicate.predicate, EmptySet):
            return Literal("")
        else:
            return Not(predicate.predicate)
    else:
        return predicate



def evaluateComplete(predicate):
    predicate = convertToDNF(predicate)
    #print("DNF", printsExpr(predicate))
    return evaluate(predicate)
def printsExpr(expr):
    if isinstance(expr, Literal):
        if (expr.value == ""):
            return "TRUE"
        return expr.value
    if isinstance(expr, CharVar):
        return "CHAR(" + expr.name + ")"
    elif isinstance(expr, EmptySet):
        return "FALSE"
    elif isinstance(expr, Equals):
        return printsExpr(expr.left) + " == " + printsExpr(expr.right)
    elif isinstance(expr, UnionPred):
        #print(expr.predicates)
        ret = ""
        for i in expr.predicates:
            ret += "(" + printsExpr(i) + ")" + " OR "
        return ret[:-3]
    elif isinstance(expr, EqualLength):
        return "|" + printsExpr(expr.left) + "| == "+ str(expr.right)
    elif isinstance(expr, StringVar):
        return "STR(" + expr.name + ")"
    elif isinstance(expr, StringIndex):
        return "STR(" + expr.stringVar.name + ")[" + str(expr.index) + "]" 
        #return "(" + printExpr(expr.left) + ") OR (" + printExpr(expr.right) + ")"
    elif isinstance(expr, AndPred):
        ret = ""
        #print(expr.predicates)
        for i in expr.predicates:
            #print(i)
            ret += "(" + printsExpr(i) + ")" + " AND "
        return ret[:-4]
    elif isinstance(expr, Not):
        return "NOT (" + printsExpr(expr.predicate) + ")"
    return expr


def distributeOrs(predicates):
    distributed = []
    for pred in predicates:
        if isinstance(pred, UnionPred):
            distributed.append(pred.predicates)
        else:
            distributed.append([pred])
    product = list(itertools.product(*distributed))
    
    # Flatten each AND group before creating final DNF
    dnf_result = []
    for group in product:
        flattened_group = []
        for pred in group:
            if isinstance(pred, AndPred):
                flattened_group.extend(pred.predicates)
            else:
                flattened_group.append(pred)
        dnf_result.append(AndPred(*flattened_group))
    
    return UnionPred(*dnf_result) if len(dnf_result) > 1 else dnf_result[0]



simplified = evaluate(UnionPred(Equals(CharVar("c1"), Literal("B")),  Equals(CharVar("c2"), Literal("C")),  Not(Equals(CharVar("c3"), Literal("E")))))
#print(printExpr(simplified))
#print(printExpr(convertToDNF(AndPred(UnionPred(Equals(Literal("A"), CharVar("c")), Equals(Literal("d"), CharVar("c"))),  UnionPred(Equals(Literal("b"), CharVar("c")), Equals(Literal("l"), CharVar("c")))  ))))


expr = UnionPred(AndPred(Equals(CharVar("c1"), Literal("c")), Literal("")), AndPred(Equals(CharVar("c1"), Literal("d")), EmptySet())) 
#print(printsExpr(expr))
simplified = evaluate(expr)

#print(printsExpr(simplified))






