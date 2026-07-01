# CLAUDE.md — Guía de estilo para la memoria del TFG

Este fichero establece las normas de estilo que debe seguir el texto de la memoria. Aplica a todos los ficheros `.tex` del directorio `report/`.

## Capitalización

Capitalizar únicamente la primera palabra de cada título. El resto va en minúsculas salvo que sea un nombre propio o una sigla (por ejemplo, "Fundamentos del sistema de tipos", no "Fundamentos del Sistema de Tipos").

## Vocabulario en español

Priorizar siempre el término español sobre el anglicismo o la sigla inglesa. Algunos ejemplos concretos de sustitución habitual:

- "prototipo" en lugar de "MVP"
- "inferencia de tipos" en lugar de "type inference"
- "árbol sintáctico" en lugar de "AST"
- "compilador" en lugar de "compiler"

Cuando sea inevitable usar una sigla en inglés, añadir su expansión completa la primera vez que aparece en el documento, entre paréntesis y seguida del término, por ejemplo: "árbol sintáctico abstracto (AST, del inglés *Abstract Syntax Tree*)".

## Negrita

No usar negrita en el cuerpo del texto. La negrita solo puede aparecer donde LaTeX la introduce por defecto (títulos de sección, cabeceras de tabla, etc.).

## Referencias a otras secciones

Cuando se mencione otra sección, capítulo, figura o tabla, usar siempre una referencia LaTeX (`\ref{}`, `\autoref{}` o `\cref{}`). No sustituir la referencia por texto plano como "en la sección anterior" o "como se describe más adelante".

## Listas

Usar listas (itemize, enumerate) solo cuando se enumeren conceptos claramente relacionados entre sí. Si se habla de dos ideas contrapuestas, de una causa y su efecto, o de cualquier relación que pueda expresarse con conectores textuales, escribirlo en párrafos continuos usando conectores como "sin embargo", "por otro lado", "además", "en cambio", "mientras que", etc.

## Aclaraciones

No usar guiones (—, –, -) para intercalar aclaraciones dentro de una oración. Sustituir por "es decir" o por paréntesis según conveya mejor el registro. Por ejemplo, escribir "el proceso de unificación (es decir, la resolución de restricciones de tipo)" en lugar de "el proceso de unificación — resolución de restricciones de tipo".

## Dos puntos

No usar dos puntos (:) fuera de enumeraciones. En particular, evitarlos en títulos, en aclaraciones dentro de párrafos y en transiciones como "cabe destacar que:". Si la oración lo requiere, reformularla o usar una coma.

## Caption de tablas

El `\caption` (y el `\label` asociado) de una tabla va siempre antes del entorno `tabular`/`tabularx`, justo después de `\centering`. Nunca va debajo del `\end{tabular}`.

## Listados de código

Todo entorno `lstlisting` debe incluir la opción `caption=` con una descripción breve en español. Esto es obligatorio para que el listado aparezca en el índice de códigos (`\lstlistoflistings`), que se genera automáticamente en el frontmatter. El caption va en la parte inferior del bloque (configurado globalmente con `captionpos=b`). No usar dos puntos en el texto del caption.

## Estado de la implementación frente a la memoria

La memoria se redacta algo adelantada respecto al estado real del código, ya que la implementación se completa durante la misma semana en que se escribe cada capítulo. Al describir las siguientes fases del compilador, redactar en presente y como si estuvieran terminadas al nivel indicado, sin usar frases como "en desarrollo", "trabajo en curso" o "parcialmente soportado" salvo que se indique explícitamente lo contrario en el futuro:

- Sobrecarga de funciones discriminada por tipo (`\autoref{sec:overloading}` en `06-capítulo6.tex`), presentar como implementada por completo.
- Aplanador perezoso (`\autoref{sec:inliner}` en `05-capítulo5.tex`), presentar como una implementación experimental o semifuncional, sin necesidad de garantizar cobertura completa de todos los casos.
- Generación de código (`\autoref{sec:codegen}` en `05-capítulo5.tex`), presentar como una base suficiente para realizar demostraciones, no como un generador de código completo o listo para producción.
