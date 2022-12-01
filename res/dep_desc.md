### 关键字说明  
value表达式中的配置项，都是当前配置项的父项  
* rev_select： 强选择  
  + select语句中的依赖
* dep：描述当前config是否可配置
  + 组关系：if ，menu的depends和visibile，choice的prompt、depends、type
  + 配置项依赖：type，prompt， depends
* restrict：对config取值限制的依赖
  + default，imply和range，以及choice组内取值限制

### 依赖关系说明
* rev_select
  + 若父项表达式取值为y，则当前项（子项）取值必须为y
  + 父项取值决定了子项的下限
* dep
  + 若子项取值为y/m，则父项表达式取值必须为y/m
  + 父项取值决定了子项的上限
* restrict
  + range（强限制），取值范围，用于int、hex配置项
  + default（弱限制），dep不成立时配置项不可配置，默认值生效
  + imply（弱限制）

### 表达式计算
* bool、tristate类型  
  + 取值为y，表达式计算中，计为2
  + 取值为m，表达式计算中，计为1
  + 取值为n，表达式计算中，计为0
* 其它类型配置项
  + 计为0
* 表达式判断  
  + 考虑优先级  
  + "!"运算符，按2-expr计算
  + "&&"运算符，取最小值  
  + "||"运算符，取最大值